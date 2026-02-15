use crate::app::main_window::MainWindowMessage;
use crate::app::plot_window::PlotWindowMessage;
use crate::app::settings::Settings;
use crate::app::{exit_confirmation_modal, layout, main_window, plot_window};
use crate::collectors::cpu_data::CpuData;
use crate::collectors::lhm_collector::{initialize_gpus, lhm_cpu_queries, lhm_gpu_queries};
use crate::collectors::{CpuCoreLHMQuery, GpuData, GpuLHMQuery};
use crate::connect_to_lhm_service;
use crate::types::{ComponentType, HardwareLogEntry, TempUnits};
use crate::utils::{csv_logger::CsvLogger, tray};
use colored::Colorize;
use iced::widget::container;
use iced::{window, Element, Subscription, Task, Theme};
use std::time::Duration;
use sysinfo::{ProcessRefreshKind, RefreshKind, System};
use tray_icon::menu::{MenuEvent, MenuId};

#[derive(Clone)]
pub enum TempMonMessage {
    // Window Lifecycle
    WindowOpened(window::Id),
    WindowClosed(window::Id),
    // Navigation
    MainButtonPressed,
    PlotterButtonPressed,
    ShowSettingsModal,
    HideSettingsModal,
    // System Tray
    TrayEvent(MenuId),
    // Settings Modal
    ThemeChanged(Theme),
    ToggleStartWithWindows(bool),
    ToggleStartMinimized(bool),
    TempUnitSelected(TempUnits),
    TempLowThresholdChanged(String),
    TempHighThresholdChanged(String),
    UpdateIntervalChanged(f32),
    SaveSettings,
    // Close/exit modal
    CloseRequestReceived(window::Id),
    CancelExit,
    ConfirmMinimize,
    ConfirmExit,
    // Hardware Data Updates
    UpdateHardwareData,
    CpuValuesUpdated((f32, f32, Vec<CpuCoreLHMQuery>)),
    GpuValuesUpdated(Vec<GpuLHMQuery>),
    HardwareMonitorConnected(Option<lhm_client::LHMClientHandle>, Vec<GpuData>),
    // Child Component Messages
    MainWindow(MainWindowMessage),
    PlotWindow(PlotWindowMessage),
}
#[derive(Clone, Debug)]
pub enum Screen {
    Main,
    Plotter,
}

pub struct TempMon {
    window_id: Option<window::Id>,
    hw_monitor_service: Option<lhm_client::LHMClientHandle>,
    cpu_data: CpuData,
    gpu_data: Vec<GpuData>,
    system: System,
    current_screen: Screen,
    show_settings_modal: bool,
    show_exit_modal: bool,
    current_theme: Theme,
    settings: Settings,
    main_window: main_window::MainWindow,
    plot_window: plot_window::PlotWindow,
    tray_icon: tray_icon::TrayIcon,
    show_menu_id: MenuId,
    quit_menu_id: MenuId,
    csv_logger: CsvLogger,
    last_error: Option<String>,
}

impl TempMon {
    /// Update tray tooltip with live hw data
    // TODO: Temperature thresholds for icon color changes are configurable in settings
    fn update_tray_tooltip(&self) {
        let cpu_str = self.settings.format_temp(self.cpu_data.temp, 0);

        let mut tooltip = format!(
            "CPU: {} {:.0}% {:.1}W {:.0}MHz",
            cpu_str,
            self.cpu_data.usage,
            self.cpu_data.total_power_draw,
            self.cpu_data.current_frequency * 1000.0,
        );

        //  Supports only one dedicated gpu systems for now
        if let Some(gpu) = self.gpu_data.first() {
            let gpu_str = self.settings.format_temp(gpu.core_temp, 0);
            tooltip.push_str(&format!(
                "\nGPU: {} {:.0}% {:.1}W",
                gpu_str, gpu.core_load, gpu.power
            ));
        }

        // Append error message if present
        if let Some(error) = &self.last_error {
            tooltip.push_str(&format!("\n⚠ Error: {}", error));
        }

        if let Err(e) = self.tray_icon.set_tooltip(Some(&tooltip)) {
            eprintln!("Failed to update tray tooltip: {}", e);
        }
    }

    pub fn new() -> (Self, Task<TempMonMessage>) {
        // Window and load program settings
        let window_settings = window::Settings {
            size: iced::Size::new(800.0, 700.0),
            position: window::Position::Centered,
            min_size: Some(iced::Size::new(500.0, 400.0)),
            icon: window::icon::from_file("assets/logo.ico").ok(),
            resizable: true,
            decorations: true,
            level: window::Level::Normal,
            exit_on_close_request: false,
            ..Default::default()
        };
        let settings = Settings::load().expect("Error loading settings");
        let current_theme = settings.theme.clone();
        let csv_logger = CsvLogger::new(None).expect("Failed to create CSV logger");
        let (_, open_task) = if settings.start_minimized {
            (window::Id::unique(), Task::none())
        } else {
            window::open(window_settings)
        };
        let (show_id, quit_id, tray_icon) = tray::init_icon(); // tray icon
        let mut system = System::new_all();
        system.refresh_cpu_all();
        system.refresh_specifics(
            RefreshKind::nothing().with_processes(
                ProcessRefreshKind::everything()
                    .without_cwd()
                    .without_environ()
                    .without_user()
                    .without_exe(),
            ),
        );
        let cpu_data = CpuData::new(&system);
        let hw_monitor_service = None;
        let plot_window = plot_window::PlotWindow::new(
            settings
                .selected_temp_units
                .as_ref()
                .map(|u| u.to_string())
                .unwrap_or_else(|| "C".to_string()),
        );

        // Create task to connect to hardware monitor
        let connect_task = Task::future(async {
            let client = connect_to_lhm_service().await;

            // Initialize GPUs if connection succeeded
            let gpu_list = if let Some(ref c) = client {
                initialize_gpus(c).await
            } else {
                Vec::new()
            };

            TempMonMessage::HardwareMonitorConnected(client, gpu_list)
        });

        (
            Self {
                window_id: None,
                hw_monitor_service,
                cpu_data,
                gpu_data: Vec::new(),
                system,
                current_screen: Screen::Main,
                show_settings_modal: false,
                show_exit_modal: false,
                current_theme,
                settings,
                main_window: main_window::MainWindow::new(),
                plot_window,
                tray_icon,
                show_menu_id: show_id,
                quit_menu_id: quit_id,
                csv_logger,
                last_error: None,
            },
            Task::batch(vec![
                // Batch tasks to run in parallel
                open_task.map(TempMonMessage::WindowOpened),
                connect_task,
            ]),
        )
    }

    pub fn theme(&self, _window: window::Id) -> Theme {
        self.current_theme.clone()
    }

    pub fn update(&mut self, message: TempMonMessage) -> Task<TempMonMessage> {
        match message {
            TempMonMessage::HardwareMonitorConnected(client, gpu_list) => {
                self.hw_monitor_service = client;
                self.gpu_data = gpu_list;

                if self.hw_monitor_service.is_some() {
                    println!("{}", "✓ Connected to hardware monitor".green());

                    if !self.gpu_data.is_empty() {
                        println!("✓ Initialized {} GPU(s)", self.gpu_data.len());
                        for (i, gpu) in self.gpu_data.iter().enumerate() {
                            println!("  GPU {}: {} ({:?})", i, gpu.name, gpu.brand);
                        }
                    }

                    // Trigger initial update after service connects
                    Task::done(TempMonMessage::UpdateHardwareData)
                } else {
                    Task::none()
                }
            }
            TempMonMessage::WindowOpened(id) => {
                self.window_id = Some(id);
                Task::none()
            }
            TempMonMessage::WindowClosed(_id) => {
                dbg!("Window closed, daemon still running...");
                self.window_id = None;
                // Flush any pending CSV logs
                if let Err(e) = self.csv_logger.flush_buffer() {
                    eprintln!("Failed to flush CSV on window close: {}", e);
                }
                Task::none()
            }
            TempMonMessage::CloseRequestReceived(id) => {
                dbg!("Close request received for window {}", id);
                self.show_exit_modal = true;
                Task::none()
            }
            TempMonMessage::CancelExit => {
                self.show_exit_modal = false;
                Task::none()
            }
            TempMonMessage::ConfirmMinimize => {
                self.show_exit_modal = false;
                if let Some(id) = self.window_id.take() {
                    return window::close(id);
                }
                Task::none()
            }
            TempMonMessage::ConfirmExit => {
                // Flush logs and kill process
                if let Err(e) = self.csv_logger.flush_buffer() {
                    eprintln!("Failed to flush CSV: {}", e);
                }
                std::process::exit(0);
            }
            TempMonMessage::TrayEvent(menu_id) => {
                if menu_id == self.show_menu_id {
                    // If window is closed, reopen it
                    if self.window_id.is_none() {
                        let window_settings = window::Settings {
                            size: iced::Size::new(800.0, 700.0),
                            position: window::Position::Centered,
                            min_size: Some(iced::Size::new(500.0, 400.0)),
                            icon: window::icon::from_file("assets/logo.ico").ok(),
                            exit_on_close_request: false,
                            ..Default::default()
                        };
                        let (_, open_task) = window::open(window_settings);
                        return open_task.map(TempMonMessage::WindowOpened);
                    }
                    Task::none()
                } else if menu_id == self.quit_menu_id {
                    // Flush CSV buffer before quitting
                    if let Err(e) = self.csv_logger.flush_buffer() {
                        eprintln!("Failed to flush CSV on quit: {}", e);
                    }
                    std::process::exit(0);
                } else {
                    Task::none()
                }
            }
            TempMonMessage::ThemeChanged(theme) => {
                self.settings.theme = theme.clone();
                Task::none()
            }
            TempMonMessage::ToggleStartWithWindows(enabled) => {
                self.settings.start_with_windows = enabled;
                Task::none()
            }
            TempMonMessage::ToggleStartMinimized(enabled) => {
                self.settings.start_minimized = enabled;
                Task::none()
            }
            TempMonMessage::TempUnitSelected(unit) => {
                // When user changes temperature unit, convert all threshold values
                if let Some(old_unit) = self.settings.selected_temp_units {
                    self.settings.temp_low_threshold =
                        old_unit.convert(self.settings.temp_low_threshold, unit);
                    self.settings.temp_high_threshold =
                        old_unit.convert(self.settings.temp_high_threshold, unit);

                    // Update the input fields to show the converted values
                    self.settings.temp_low_input =
                        format!("{:.0}", self.settings.temp_low_threshold);
                    self.settings.temp_high_input =
                        format!("{:.0}", self.settings.temp_high_threshold);
                }

                self.settings.selected_temp_units = Option::from(unit);
                Task::none()
            }
            TempMonMessage::TempLowThresholdChanged(value) => {
                self.settings.temp_low_input = value;
                Task::none()
            }
            TempMonMessage::TempHighThresholdChanged(value) => {
                self.settings.temp_high_input = value;
                Task::none()
            }
            TempMonMessage::UpdateIntervalChanged(value) => {
                self.settings.data_update_interval = value;
                self.settings.update_interval_input = value.to_string();
                Task::none()
            }
            TempMonMessage::SaveSettings => {
                // Parse and validate temperature thresholds
                if let Ok(low) = self.settings.temp_low_input.parse::<f32>() {
                    if let Ok(high) = self.settings.temp_high_input.parse::<f32>() {
                        if low < high {
                            // Store thresholds in the selected unit (no conversion)
                            self.settings.temp_low_threshold = low;
                            self.settings.temp_high_threshold = high;
                            self.current_theme = self.settings.theme.clone();
                        }
                    }
                }
                Settings::save(&self.settings).expect("Error saving settings");
                if let Err(e) = crate::utils::startup::set_start_with_windows(self.settings.start_with_windows) {
                    eprintln!("Failed to update startup registry: {}", e);
                }
                self.show_settings_modal = false;
                Task::none()
            }
            TempMonMessage::MainButtonPressed => {
                self.current_screen = Screen::Main;
                Task::none()
            }
            TempMonMessage::PlotterButtonPressed => {
                self.current_screen = Screen::Plotter;
                Task::none()
            }
            TempMonMessage::ShowSettingsModal => {
                // Reset input fields to current saved values when opening modal
                self.settings.temp_low_input = self.settings.temp_low_threshold.to_string();
                self.settings.temp_high_input = self.settings.temp_high_threshold.to_string();
                self.show_settings_modal = true;
                Task::none()
            }

            TempMonMessage::HideSettingsModal => {
                self.show_settings_modal = false;
                Task::none()
            }
            TempMonMessage::MainWindow(msg) => {
                self.main_window.update(msg);
                Task::none()
            }
            TempMonMessage::PlotWindow(msg) => {
                let task = self.plot_window.update(
                    &self.csv_logger,
                    msg,
                    &self.system,
                    self.settings.selected_temp_units.unwrap(),
                    &self.gpu_data,
                );
                task.map(TempMonMessage::PlotWindow)
            }
            TempMonMessage::UpdateHardwareData => {
                self.cpu_data.update(&mut self.system);
                self.system.refresh_specifics(
                    RefreshKind::nothing().with_processes(
                        ProcessRefreshKind::everything()
                            .without_cwd()
                            .without_environ()
                            .without_user()
                            .without_exe(),
                    ),
                );
                if let Some(client) = &self.hw_monitor_service {
                    let client_cpu = client.clone();
                    let client_gpu = client.clone();
                    let gpu_brands: Vec<_> = self.gpu_data.iter().map(|gpu| gpu.brand).collect();

                    Task::batch(vec![
                        // CPU Task: Inline the update and query, handle error at the end
                        Task::future(async move {
                            // Wrap multiple async steps in an inner block to chain errors with '?'
                            let result = async {
                                client_cpu.update_all().await?;
                                lhm_cpu_queries(&client_cpu).await
                            }
                            .await;

                            // Map success to Message, or handle error with unwrap_or_else
                            result
                                .map(TempMonMessage::CpuValuesUpdated)
                                .unwrap_or_else(|e| {
                                    eprintln!("Failed to query CPU: {}", e);
                                    TempMonMessage::CpuValuesUpdated((0.0, 0.0, Vec::new()))
                                })
                        }),
                        // GPU Task: Inline the iteration
                        Task::future(async move {
                            let result = async {
                                let mut queries = Vec::new();
                                for brand in gpu_brands {
                                    // Using '?' here propagates the error immediately, matching original behavior
                                    queries.push(lhm_gpu_queries(brand, &client_gpu).await?);
                                }
                                Ok(queries)
                            }
                            .await;

                            result.map(TempMonMessage::GpuValuesUpdated).unwrap_or_else(
                                |e: anyhow::Error| {
                                    eprintln!("Failed to query GPU: {}", e);
                                    TempMonMessage::GpuValuesUpdated(Vec::new())
                                },
                            )
                        }),
                    ])
                } else {
                    Task::none()
                }
            }
            TempMonMessage::CpuValuesUpdated(temps) => {
                // Collect everything from lhm queries into CpuData
                self.cpu_data.update_lhm_data(temps);
                // Update tray tooltip with fresh hardware data
                self.update_tray_tooltip();

                // Convert temperature to user's selected unit for CSV logging
                let selected_unit = self.settings.temp_unit();
                let converted_temp = TempUnits::Celsius.convert(self.cpu_data.temp, selected_unit);

                // Log CPU data to CSV
                let entry = HardwareLogEntry {
                    timestamp: chrono::Local::now().to_rfc3339(),
                    selected_process: self.plot_window.format_selected_processes_for_csv(), // No system arg needed, uses cache
                    component_type: ComponentType::CPU,
                    model_name: self.cpu_data.name.clone(),
                    temperature_unit: selected_unit.to_string(),
                    temperature: converted_temp,
                    usage: self.cpu_data.usage,
                    power_draw: self.cpu_data.total_power_draw,
                };

                match self.csv_logger.write(vec![entry]) {
                    Ok(_) => {
                        // Clear error on successful write
                        self.last_error = None;
                    }
                    Err(e) => {
                        let error_msg = format!("CSV write failed: {}", e);
                        eprintln!("{}", error_msg);
                        self.last_error = Some(error_msg);
                    }
                }

                let _ = self.plot_window.update(
                    &self.csv_logger,
                    PlotWindowMessage::RefreshData,
                    &self.system,
                    self.settings
                        .selected_temp_units
                        .unwrap_or(TempUnits::Celsius),
                    &self.gpu_data,
                );
                Task::none()
            }
            TempMonMessage::GpuValuesUpdated(gpu_queries) => {
                // Update each GPU with its corresponding query data
                for (i, query) in gpu_queries.into_iter().enumerate() {
                    if let Some(gpu) = self.gpu_data.get_mut(i) {
                        gpu.update_lhm_data(query);

                        // Convert temperature to user's selected unit for CSV logging
                        let selected_unit = self.settings.temp_unit();
                        let converted_temp =
                            TempUnits::Celsius.convert(self.gpu_data[i].core_temp, selected_unit);

                        // Log CPU data to CSV
                        let entry = HardwareLogEntry {
                            timestamp: chrono::Local::now().to_rfc3339(),
                            selected_process: self.plot_window.format_selected_processes_for_csv(), // No system arg needed, uses cache
                            component_type: ComponentType::GPU,
                            model_name: self.gpu_data[i].name.clone(),
                            temperature_unit: selected_unit.to_string(),
                            temperature: converted_temp,
                            usage: self.gpu_data[i].core_load,
                            power_draw: self.gpu_data[i].power,
                        };

                        match self.csv_logger.write(vec![entry]) {
                            Ok(_) => {
                                // Clear error on successful write
                                self.last_error = None;
                            }
                            Err(e) => {
                                let error_msg = format!("CSV write failed: {}", e);
                                eprintln!("{}", error_msg);
                                self.last_error = Some(error_msg);
                            }
                        }
                        let _ = self.plot_window.update(
                            &self.csv_logger,
                            PlotWindowMessage::RefreshData,
                            &self.system,
                            self.settings
                                .selected_temp_units
                                .unwrap_or(TempUnits::Celsius),
                            &self.gpu_data,
                        );
                    }
                }
                Task::none()
            }
        }
    }

    pub fn view(&self, window_id: window::Id) -> Element<'_, TempMonMessage> {
        if self.window_id != Some(window_id) {
            return container("").into();
        }
        let page = match self.current_screen {
            Screen::Main => self
                .main_window
                .view(&self.cpu_data, &self.gpu_data, &self.settings)
                .map(TempMonMessage::MainWindow),
            Screen::Plotter => self.plot_window.view().map(TempMonMessage::PlotWindow),
        };
        if self.show_settings_modal {
            self.settings
                .view(layout::with_header(page, &self.current_screen))
        } else if self.show_exit_modal {
            exit_confirmation_modal::exit_confirmation_modal(layout::with_header(
                page,
                &self.current_screen,
            ))
        } else {
            layout::with_header(page, &self.current_screen)
        }
    }

    pub fn subscription(&self) -> Subscription<TempMonMessage> {
        // https://docs.iced.rs/iced/#passive-subscriptions
        Subscription::batch(vec![
            window::close_requests().map(TempMonMessage::CloseRequestReceived),
            iced::time::every(Duration::from_secs_f32(self.settings.data_update_interval))
                .map(|_| TempMonMessage::UpdateHardwareData),
            tray_events_subscription(),
            self.plot_window
                .subscription()
                .map(TempMonMessage::PlotWindow),
            self.main_window
                .subscription()
                .map(TempMonMessage::MainWindow),
        ])
    }
}

/// Subscription for tray menu events
fn tray_events_subscription() -> Subscription<TempMonMessage> {
    use iced::futures::SinkExt;

    Subscription::run(|| {
        iced::stream::channel(
            50,
            |mut output: iced::futures::channel::mpsc::Sender<TempMonMessage>| async move {
                loop {
                    tokio::time::sleep(Duration::from_millis(50)).await;

                    // Poll menu events from tray-icon
                    while let Ok(event) = MenuEvent::receiver().try_recv() {
                        let _ = output.send(TempMonMessage::TrayEvent(event.id)).await;
                    }
                }
            },
        )
    })
}

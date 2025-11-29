#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide terminal on Windows
mod app;
mod assets;
mod collectors;
mod utils;

use app::plot_window;
use app::plot_window::PlotWindowMessage;
use app::settings::Settings;
use app::{layout, main_window};
use collectors::lhm_collector::{initialize_gpus, lhm_cpu_queries, lhm_gpu_queries};
use collectors::{cpu_collector::CpuData, gpu_collector::GpuData};
use collectors::{CpuCoreLHMQuery, GpuLHMQuery};
use colored::Colorize;
use iced::widget::container;
use iced::{window, Element, Subscription, Task, Theme};
use lhm_client::service::is_service_installed;
use lhm_client::{ComputerOptions, LHMClient};
use std::time::Duration;
use sysinfo::System;
use tray_icon::{
    menu::{Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem},
    Icon, TrayIconBuilder,
};
use utils::csv_logger::{CsvCpuLogEntry, CsvLogger};
async fn connect_to_lhm_service() -> Option<lhm_client::LHMClientHandle> {
    match LHMClient::connect().await {
        Ok(client) => {
            println!("Connected to hardware monitoring service");
            client
                .set_options(ComputerOptions {
                    controller_enabled: false,
                    cpu_enabled: true,
                    gpu_enabled: true,
                    motherboard_enabled: false,
                    battery_enabled: false,
                    memory_enabled: false,
                    network_enabled: false,
                    psu_enabled: true,
                    storage_enabled: false,
                })
                .await
                .unwrap();
            client.update_all().await.unwrap();
            println!("{}", "Service options set".green().bold());
            Some(client)
        }
        Err(e) => {
            eprintln!("{} {}", "Failed to connect to service: {}".red(), e);
            eprintln!("{}", "The service may not be running. Try:".red());
            eprintln!("{}", "1. Run 'install-service.bat' as administrator".red());
            eprintln!(
                "{}",
                "2. Or manually start the service from Services (services.msc)".red()
            );
            None
        }
    }
}

fn main() -> iced::Result {
    match is_service_installed() {
        Ok(true) => {
            println!("{}", "✓ Service is ready".green());
        }
        Ok(false) => {
            eprintln!(
                "{}",
                "Hardware monitoring service not installed".red().bold()
            );
            eprintln!(
                "{}",
                "Please run 'install-service.bat' as administrator"
                    .red()
                    .bold()
            );
            // TODO: Show user a dialog or instructions
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Error checking service: {}", e);
            std::process::exit(1);
        }
    }
    iced::daemon(App::new, App::update, App::view)
        .subscription(App::subscription)
        .title("TempMon")
        .antialiasing(true)
        .theme(App::theme)
        .run()
}

#[derive(Clone)]
enum AppMessage {
    WindowOpened(window::Id),
    WindowClosed(window::Id),
    TrayEvent(MenuId),
    ShowSettingsModal,
    HideSettingsModal,
    ThemeChanged(Theme),
    ToggleStartWithWindows(bool),
    ToggleStartMinimized(bool),
    TempUnitSelected(app::settings::TempUnits),
    TempLowThresholdChanged(String),
    TempHighThresholdChanged(String),
    UpdateIntervalChanged(f32),
    SaveSettings,
    MainButtonPressed,
    PlotterButtonPressed,
    UpdateHardwareData,
    CpuValuesUpdated((f32, f32, Vec<CpuCoreLHMQuery>)),
    GpuValuesUpdated(Vec<GpuLHMQuery>),
    MainWindow(main_window::MainWindowMessage),
    PlotWindow(PlotWindowMessage),
    HardwareMonitorConnected(Option<lhm_client::LHMClientHandle>, Vec<GpuData>),
}
#[derive(Clone, Debug)]
enum Screen {
    Main,
    Plotter,
}

struct App {
    window_id: Option<window::Id>,
    hw_monitor_service: Option<lhm_client::LHMClientHandle>,
    cpu_data: CpuData,
    gpu_data: Vec<GpuData>,
    system: System,
    current_screen: Screen,
    show_settings_modal: bool,
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

impl App {
    /// Update tray tooltip with live hw data
    // TODO: Temperature thresholds for icon color changes are configurable in settings
    fn update_tray_tooltip(&self) {
        let mut tooltip = format!(
            "CPU: {:.0}°C ({:.0}%)\nPower: {:.1}W",
            self.cpu_data.temp, self.cpu_data.usage, self.cpu_data.total_power_draw
        );

        // Append error message if present
        if let Some(error) = &self.last_error {
            tooltip.push_str(&format!("\n⚠ Error: {}", error));
        }

        if let Err(e) = self.tray_icon.set_tooltip(Some(&tooltip)) {
            eprintln!("Failed to update tray tooltip: {}", e);
        }
    }

    fn new() -> (Self, Task<AppMessage>) {
        let window_settings = window::Settings {
            size: iced::Size::new(800.0, 700.0),
            position: window::Position::Centered,
            min_size: Some(iced::Size::new(500.0, 400.0)),
            icon: window::icon::from_file("assets/logo.ico").ok(),
            resizable: true,
            decorations: true,
            level: window::Level::Normal,
            ..Default::default()
        };

        let (_, open_task) = window::open(window_settings);

        // Load tray icon from bytes
        const ICON_DATA: &[u8] = include_bytes!("../assets/logo.ico");
        let image = image::load_from_memory(ICON_DATA)
            .expect("Failed to load icon from memory")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        let icon = Icon::from_rgba(rgba, width, height).expect("Failed to create icon");
        // Create tray menu
        let menu = Menu::new();
        let show_item = MenuItem::new("Show Window", true, None);
        let quit_item = MenuItem::new("Quit", true, None);
        let separator = PredefinedMenuItem::separator();

        // Store menu IDs for event handling
        let show_id = show_item.id().clone();
        let quit_id = quit_item.id().clone();

        menu.append_items(&[&show_item, &separator, &quit_item])
            .expect("Failed to append menu items");

        // Build tray icon
        let tray_icon = TrayIconBuilder::new()
            .with_tooltip("TempMon")
            .with_icon(icon)
            .with_menu(Box::new(menu))
            .build()
            .expect("Failed to create tray icon");

        let mut system = System::new_all();
        system.refresh_cpu_all();
        let cpu_data = CpuData::new(&system);
        let hw_monitor_service = None;
        let settings = Settings::load().expect("Error loading settings");
        let current_theme = settings.theme.clone();
        let csv_logger = CsvLogger::new(None).expect("Failed to create CSV logger");
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

            AppMessage::HardwareMonitorConnected(client, gpu_list)
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
                open_task.map(AppMessage::WindowOpened),
                connect_task,
            ]),
        )
    }

    fn theme(&self, _window: window::Id) -> Theme {
        self.current_theme.clone()
    }

    fn update(&mut self, message: AppMessage) -> Task<AppMessage> {
        match message {
            AppMessage::HardwareMonitorConnected(client, gpu_list) => {
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
                    Task::done(AppMessage::UpdateHardwareData)
                } else {
                    Task::none()
                }
            }
            AppMessage::WindowOpened(id) => {
                self.window_id = Some(id);
                Task::none()
            }
            AppMessage::WindowClosed(_id) => {
                dbg!("Window closed, daemon still running...");
                self.window_id = None;

                // Flush any pending CSV logs
                if let Err(e) = self.csv_logger.flush_buffer() {
                    eprintln!("Failed to flush CSV on window close: {}", e);
                }

                Task::none()
            }
            AppMessage::TrayEvent(menu_id) => {
                if menu_id == self.show_menu_id {
                    // If window is closed, reopen it
                    if self.window_id.is_none() {
                        let window_settings = window::Settings {
                            size: iced::Size::new(800.0, 700.0),
                            position: window::Position::Centered,
                            min_size: Some(iced::Size::new(500.0, 400.0)),
                            icon: window::icon::from_file("assets/logo.ico").ok(),
                            ..Default::default()
                        };
                        let (_, open_task) = window::open(window_settings);
                        return open_task.map(AppMessage::WindowOpened);
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
            AppMessage::ThemeChanged(theme) => {
                self.settings.theme = theme.clone();
                Task::none()
            }
            AppMessage::ToggleStartWithWindows(enabled) => {
                self.settings.start_with_windows = enabled;
                Task::none()
            }
            AppMessage::ToggleStartMinimized(enabled) => {
                self.settings.start_minimized = enabled;
                Task::none()
            }
            AppMessage::TempUnitSelected(unit) => {
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
            AppMessage::TempLowThresholdChanged(value) => {
                self.settings.temp_low_input = value;
                Task::none()
            }
            AppMessage::TempHighThresholdChanged(value) => {
                self.settings.temp_high_input = value;
                Task::none()
            }
            AppMessage::UpdateIntervalChanged(value) => {
                self.settings.data_update_interval = value;
                self.settings.update_interval_input = value.to_string();
                Task::none()
            }
            AppMessage::SaveSettings => {
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
                self.show_settings_modal = false;
                Task::none()
            }
            AppMessage::MainButtonPressed => {
                self.current_screen = Screen::Main;
                Task::none()
            }
            AppMessage::PlotterButtonPressed => {
                self.current_screen = Screen::Plotter;
                Task::none()
            }
            AppMessage::ShowSettingsModal => {
                // Reset input fields to current saved values when opening modal
                self.settings.temp_low_input = self.settings.temp_low_threshold.to_string();
                self.settings.temp_high_input = self.settings.temp_high_threshold.to_string();
                self.show_settings_modal = true;
                Task::none()
            }

            AppMessage::HideSettingsModal => {
                self.show_settings_modal = false;
                Task::none()
            }
            AppMessage::MainWindow(msg) => {
                self.main_window.update(msg);
                Task::none()
            }
            AppMessage::PlotWindow(msg) => {
                self.plot_window.update(
                    &self.csv_logger,
                    msg,
                    self.settings.selected_temp_units.unwrap(),
                );
                Task::none()
            }
            AppMessage::UpdateHardwareData => {
                self.cpu_data.update(&mut self.system);

                if let Some(client) = &self.hw_monitor_service {
                    let client_cpu = client.clone();
                    let client_gpu = client.clone();
                    let gpu_brands: Vec<_> = self.gpu_data.iter().map(|gpu| gpu.brand).collect();

                    Task::batch(vec![
                        // Query CPU data
                        Task::future(async move {
                            client_cpu
                                .update_all()
                                .await
                                .expect("Error updating hardware");
                            let temps = lhm_cpu_queries(&client_cpu).await;
                            AppMessage::CpuValuesUpdated(temps)
                        }),
                        // Query GPU data
                        Task::future(async move {
                            let mut gpu_queries = Vec::new();

                            for brand in gpu_brands {
                                let query = lhm_gpu_queries(brand, &client_gpu).await;
                                gpu_queries.push(query);
                            }

                            AppMessage::GpuValuesUpdated(gpu_queries)
                        }),
                    ])
                } else {
                    Task::none()
                }
            }
            AppMessage::CpuValuesUpdated(temps) => {
                // Collect everything from lhm queries into CpuData
                self.cpu_data.update_lhm_data(temps);
                // Update tray tooltip with fresh hardware data
                self.update_tray_tooltip();

                // Log CPU data to CSV
                let entry = CsvCpuLogEntry {
                    timestamp: chrono::Local::now().to_rfc3339(),
                    temperature_unit: self
                        .settings
                        .selected_temp_units
                        .as_ref()
                        .map(|u| u.to_string())
                        .unwrap_or_else(|| "C".to_string()),
                    temperature: self.cpu_data.temp,
                    cpu_usage: self.cpu_data.usage,
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
                self.plot_window.update(
                    &self.csv_logger,
                    PlotWindowMessage::Tick,
                    self.settings
                        .selected_temp_units
                        .unwrap_or(app::settings::TempUnits::Celsius),
                );
                Task::none()
            }
            AppMessage::GpuValuesUpdated(gpu_queries) => {
                // Update each GPU with its corresponding query data
                for (i, query) in gpu_queries.into_iter().enumerate() {
                    if let Some(gpu) = self.gpu_data.get_mut(i) {
                        gpu.update_lhm_data(query);
                    }
                }
                Task::none()
            }
        }
    }

    fn view(&self, window_id: window::Id) -> Element<'_, AppMessage> {
        if self.window_id != Some(window_id) {
            return container("").into();
        }
        let page = match self.current_screen {
            Screen::Main => self
                .main_window
                .view(&self.cpu_data, &self.gpu_data)
                .map(AppMessage::MainWindow),
            Screen::Plotter => self.plot_window.view().map(AppMessage::PlotWindow),
        };
        if self.show_settings_modal {
            self.settings.view(layout::with_header(page))
        } else {
            layout::with_header(page)
        }
    }

    fn subscription(&self) -> Subscription<AppMessage> {
        // https://docs.iced.rs/iced/#passive-subscriptions
        Subscription::batch(vec![
            window::close_events().map(AppMessage::WindowClosed),
            iced::time::every(Duration::from_secs_f32(self.settings.data_update_interval))
                .map(|_| AppMessage::UpdateHardwareData),
            tray_events_subscription(),
            self.plot_window.subscription().map(AppMessage::PlotWindow),
            self.main_window.subscription().map(AppMessage::MainWindow),
        ])
    }
}

/// Subscription for tray menu events
fn tray_events_subscription() -> Subscription<AppMessage> {
    use iced::futures::SinkExt;

    Subscription::run(|| {
        iced::stream::channel(
            50,
            |mut output: iced::futures::channel::mpsc::Sender<AppMessage>| async move {
                loop {
                    tokio::time::sleep(Duration::from_millis(50)).await;

                    // Poll menu events from tray-icon
                    while let Ok(event) = MenuEvent::receiver().try_recv() {
                        let _ = output.send(AppMessage::TrayEvent(event.id)).await;
                    }
                }
            },
        )
    })
}

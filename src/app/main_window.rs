use super::cards;
use crate::app::graphs::gauge::{Gauge, Placement, Zone};
use crate::app::settings::Settings;
use crate::collectors::cpu_data::CpuData;
use crate::collectors::GpuData;
use crate::types::CpuBarChartState;
use iced::widget::{column, container, scrollable};
use iced::{window, Element, Fill, Subscription};
use lilt::{Animated, Easing};
use std::time::Instant;

#[derive(Debug, Clone)]
pub enum MainWindowMessage {
    UsageButtonPressed,
    PowerButtonPressed,
    // Animation triggers
    ToggleCpuCard,
    ToggleCoresCard,
    ToggleGpuCard,
    Tick, // Frame update (REQUIRED for animations)
    GpuButtonPressed(usize),
    UpdateGaugeValue(f64), // Update gauge with new temperature
}

pub struct MainWindow {
    cpu_bar_chart_state: CpuBarChartState,
    cpu_card_expanded: Animated<f32, Instant>,
    cores_card_expanded: Animated<f32, Instant>,
    gpu_card_expanded: Animated<f32, Instant>,
    selected_gpu_index: usize,
    now: Instant,
    cpu_temp_gauge: Gauge,
}

//TODO: Check for CPU cores bar chart overflow: scrollable container?
//TODO: Responsive layout: max size for cards and move them according to screen size (switch between column/row or some better way with iced api)
//TODO: Tiling window management for cards? https://docs.iced.rs/iced_widget/pane_grid/struct.PaneGrid.html
// TODO: 1: 5 sec timeout before setting min/max values. 2: 100% max value clips with box next to it
impl MainWindow {
    pub fn new() -> Self {
        let cpu_temp_gauge = Gauge::new("CPU TEMP", 0.0, 100.0)
            .unit("°C")
            // No animation to avoid CPU usage
            .span(240.0)
            .thickness(0.75)
            .decimals(1)
            .zone(Zone::Success(60.0))
            .zone(Zone::Warning(75.0))
            .zone(Zone::Danger(100.0))
            .zone_opacity(0.3)
            .value_pos(Placement::Center)
            .title_pos(Placement::Bottom);

        Self {
            cpu_bar_chart_state: CpuBarChartState::Usage,
            cpu_card_expanded: Animated::new(1.0).duration(400.0).easing(Easing::EaseInOut),
            cores_card_expanded: Animated::new(1.0).duration(400.0).easing(Easing::EaseInOut),
            gpu_card_expanded: Animated::new(1.0).duration(400.0).easing(Easing::EaseInOut),
            selected_gpu_index: 0,
            now: Instant::now(),
            cpu_temp_gauge,
        }
    }

    pub fn update(&mut self, message: MainWindowMessage) {
        match message {
            MainWindowMessage::UsageButtonPressed => {
                self.cpu_bar_chart_state = CpuBarChartState::Usage;
            }
            MainWindowMessage::PowerButtonPressed => {
                self.cpu_bar_chart_state = CpuBarChartState::Power;
            }
            MainWindowMessage::GpuButtonPressed(index) => {
                self.selected_gpu_index = index;
            }
            MainWindowMessage::ToggleCpuCard => {
                // 0.0 Collapsed, 1.0 Expanded
                let new_value = if self.cpu_card_expanded.value > 0.5 {
                    0.0
                } else {
                    1.0
                };
                // Start the transition
                self.cpu_card_expanded.transition(new_value, Instant::now());
            }
            MainWindowMessage::ToggleCoresCard => {
                let new_value = if self.cores_card_expanded.value > 0.5 {
                    0.0
                } else {
                    1.0
                };
                self.cores_card_expanded
                    .transition(new_value, Instant::now());
            }
            MainWindowMessage::ToggleGpuCard => {
                let new_value = if self.gpu_card_expanded.value > 0.5 {
                    0.0
                } else {
                    1.0
                };
                self.gpu_card_expanded.transition(new_value, Instant::now());
            }
            MainWindowMessage::Tick => {
                // Update current time on each frame
                self.now = Instant::now();
            }
            MainWindowMessage::UpdateGaugeValue(temp) => {
                // Update gauge with new temperature
                self.cpu_temp_gauge.set_value(temp);
            }
        }
    }

    pub fn subscription(&self) -> Subscription<MainWindowMessage> {
        // Only subscribe to frames when card animations are active
        // Gauge has no animation, so no need to check it
        if self.cpu_card_expanded.in_progress(self.now)
            || self.cores_card_expanded.in_progress(self.now)
            || self.gpu_card_expanded.in_progress(self.now)
        {
            window::frames().map(|_| MainWindowMessage::Tick)
        } else {
            Subscription::none()
        }
    }

    pub fn view<'a>(
        &'a self,
        cpu_data: &'a CpuData,
        gpu_data: &'a Vec<GpuData>,
        settings: &'a Settings,
    ) -> Element<'a, MainWindowMessage> {
        // Note: Gauge value is updated via UpdateGaugeValue message from parent
        // when hardware data changes

        // Calculate animation factors
        let cpu_animation_factor = self
            .cpu_card_expanded
            .animate(std::convert::identity, self.now);
        let is_cpu_card_expanded = self.cpu_card_expanded.value > 0.5;

        let cores_animation_factor = self
            .cores_card_expanded
            .animate(std::convert::identity, self.now);
        let is_cores_expanded = self.cores_card_expanded.value > 0.5;

        let gpu_animation_factor = self
            .gpu_card_expanded
            .animate(std::convert::identity, self.now);
        let is_gpu_card_expanded = self.gpu_card_expanded.value > 0.5;

        // Render gauge chart (gauge is borrowed immutably from &self)
        let gauge_chart = self.cpu_temp_gauge.chart()
            .height(iced::Length::Fixed(160.0))
            .width(iced::Length::Fixed(180.0));

        // Render cards using extracted modules
        let cpu_card = cards::cpu_card::render_general_cpu_card(
            cpu_data,
            settings,
            cpu_animation_factor,
            is_cpu_card_expanded,
            MainWindowMessage::ToggleCpuCard,
            gauge_chart.into(),
        );

        let cores_card = cards::cpu_cores_card::render_cores_card(
            &cpu_data.core_utilization,
            &cpu_data.core_power_draw,
            self.cpu_bar_chart_state,
            cores_animation_factor,
            is_cores_expanded,
            MainWindowMessage::ToggleCoresCard,
        );

        let gpu_card = cards::gpu_card::render_gpu_card(
            gpu_data,
            settings,
            self.selected_gpu_index,
            gpu_animation_factor,
            is_gpu_card_expanded,
            MainWindowMessage::ToggleGpuCard,
        );

        // Build card layout
        let mut all_cards = column![cpu_card, cores_card].spacing(20);
        if let Some(gpu) = gpu_card {
            all_cards = all_cards.push(gpu);
        }

        scrollable(container(all_cards).padding(20).width(Fill)).into()
    }
}

//! Adaptive Dashboard
//!
//! A dynamic dashboard that displays system status, metrics, and adaptive suggestions.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use iced::widget::{button, column, container, row, text, progress_bar, Space};
use iced::{Element, Length, Subscription};

use super::adaptive::AdaptiveState;
use super::theme::KolibriTheme;
use super::Message;

/// Dashboard widget
#[derive(Debug)]
pub struct Dashboard {
    /// Current dashboard data
    data: DashboardData,
    
    /// Update history for trends
    history: VecDeque<DashboardSnapshot>,
    
    /// Last update time
    last_update: Instant,
    
    /// Animation state
    animation_state: DashboardAnimation,
    
    /// Selected panel
    selected_panel: DashboardPanel,
    
    /// Refresh interval
    refresh_interval: Duration,
}

/// Dashboard data
#[derive(Debug, Clone, Default)]
pub struct DashboardData {
    /// System metrics
    pub system: SystemMetrics,
    
    /// Memory cell status
    pub memory_cell: CellStatus,
    
    /// Processor cell status
    pub processor_cell: CellStatus,
    
    /// Neural scheduler status
    pub neural_scheduler: NeuralSchedulerStatus,
    
    /// Unified Mind status
    pub unified_mind: UnifiedMindStatus,
    
    /// Active alerts
    pub alerts: Vec<Alert>,
    
    /// Suggestions from Unified Mind
    pub suggestions: Vec<Suggestion>,
    
    /// Recent activities
    pub activities: VecDeque<Activity>,
}

/// System metrics
#[derive(Debug, Clone)]
pub struct SystemMetrics {
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub disk_usage: f32,
    pub network_usage: f32,
    pub uptime_seconds: u64,
    pub process_count: u32,
    pub thread_count: u32,
    pub load_average: [f32; 3],
}

impl Default for SystemMetrics {
    fn default() -> Self {
        Self {
            cpu_usage: 0.0,
            memory_usage: 0.0,
            disk_usage: 0.0,
            network_usage: 0.0,
            uptime_seconds: 0,
            process_count: 0,
            thread_count: 0,
            load_average: [0.0, 0.0, 0.0],
        }
    }
}

/// Cell status
#[derive(Debug, Clone)]
pub struct CellStatus {
    pub name: String,
    pub state: CellState,
    pub health: HealthStatus,
    pub last_heartbeat: Option<Instant>,
    pub metrics: CellMetrics,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellState {
    Initializing,
    Active,
    Degraded,
    Healing,
    Shutdown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Default)]
pub struct CellMetrics {
    pub operations_total: u64,
    pub operations_per_second: f32,
    pub errors_total: u64,
    pub avg_latency_ms: f32,
}

impl Default for CellStatus {
    fn default() -> Self {
        Self {
            name: String::new(),
            state: CellState::Initializing,
            health: HealthStatus::Healthy,
            last_heartbeat: None,
            metrics: CellMetrics::default(),
        }
    }
}

/// Neural scheduler status
#[derive(Debug, Clone)]
pub struct NeuralSchedulerStatus {
    pub active: bool,
    pub current_decision: String,
    pub confidence: f32,
    pub decisions_today: u64,
    pub avg_confidence: f32,
}

impl Default for NeuralSchedulerStatus {
    fn default() -> Self {
        Self {
            active: false,
            current_decision: String::new(),
            confidence: 0.0,
            decisions_today: 0,
            avg_confidence: 0.0,
        }
    }
}

/// Unified Mind status
#[derive(Debug, Clone)]
pub struct UnifiedMindStatus {
    pub state: String,
    pub llm_provider: String,
    pub queries_processed: u64,
    pub uptime: String,
    pub context_items: u32,
}

impl Default for UnifiedMindStatus {
    fn default() -> Self {
        Self {
            state: "Unknown".to_string(),
            llm_provider: "None".to_string(),
            queries_processed: 0,
            uptime: "N/A".to_string(),
            context_items: 0,
        }
    }
}

/// Alert
#[derive(Debug, Clone)]
pub struct Alert {
    pub severity: AlertSeverity,
    pub title: String,
    pub message: String,
    pub timestamp: Instant,
    pub source: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Suggestion from Unified Mind
#[derive(Debug, Clone)]
pub struct Suggestion {
    pub suggestion_type: SuggestionType,
    pub title: String,
    pub description: String,
    pub action: String,
    pub confidence: f32,
    pub relevance: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SuggestionType {
    Performance,
    Memory,
    Security,
    Productivity,
    Creative,
}

/// Activity record
#[derive(Debug, Clone)]
pub struct Activity {
    pub activity_type: String,
    pub description: String,
    pub timestamp: Instant,
    pub impact: ActivityImpact,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivityImpact {
    Low,
    Medium,
    High,
}

/// Dashboard snapshot for history
#[derive(Debug, Clone)]
pub struct DashboardSnapshot {
    pub timestamp: Instant,
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub network_usage: f32,
}

/// Dashboard animation state
#[derive(Debug, Clone, Default)]
pub struct DashboardAnimation {
    pub pulse_phase: f32,
    pub slide_offset: f32,
    pub fade_alpha: f32,
}

/// Dashboard panel selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DashboardPanel {
    Overview,
    System,
    Cells,
    NeuralScheduler,
    Mind,
    Activities,
}

impl Default for Dashboard {
    fn default() -> Self {
        Self::new()
    }
}

impl Dashboard {
    pub fn new() -> Self {
        Self {
            data: DashboardData::default(),
            history: VecDeque::with_capacity(100),
            last_update: Instant::now(),
            animation_state: DashboardAnimation::default(),
            selected_panel: DashboardPanel::Overview,
            refresh_interval: Duration::from_millis(1000),
        }
    }
    
    /// Update dashboard data
    pub fn update(&mut self, data: DashboardData) {
        // Add snapshot to history
        self.history.push_back(DashboardSnapshot {
            timestamp: Instant::now(),
            cpu_usage: data.system.cpu_usage,
            memory_usage: data.system.memory_usage,
            network_usage: data.system.network_usage,
        });
        
        // Keep only last 100 snapshots
        while self.history.len() > 100 {
            self.history.pop_front();
        }
        
        self.data = data;
        self.last_update = Instant::now();
    }
    
    /// Tick animation
    pub fn tick(&mut self) {
        self.animation_state.pulse_phase += 0.05;
        if self.animation_state.pulse_phase > std::f32::consts::TAU {
            self.animation_state.pulse_phase -= std::f32::consts::TAU;
        }
    }
    
    /// Get subscription for dashboard updates
    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::none() // Will be implemented with actual data source
    }
    
    /// Render dashboard view
    pub fn view(&self) -> Element<'_, Message> {
        let theme = KolibriTheme::default();
        
        // Panel selector
        let panel_selector = self.render_panel_selector(&theme);
        
        // Main content based on selected panel
        let main_content = match self.selected_panel {
            DashboardPanel::Overview => self.render_overview(&theme),
            DashboardPanel::System => self.render_system_panel(&theme),
            DashboardPanel::Cells => self.render_cells_panel(&theme),
            DashboardPanel::NeuralScheduler => self.render_neural_scheduler_panel(&theme),
            DashboardPanel::Mind => self.render_mind_panel(&theme),
            DashboardPanel::Activities => self.render_activities_panel(&theme),
        };
        
        column![
            panel_selector,
            Space::with_height(10),
            main_content,
        ]
        .spacing(10)
        .into()
    }
    
    fn render_panel_selector(&self, theme: &KolibriTheme) -> Element<'_, Message> {
        let panels = [
            (DashboardPanel::Overview, "Overview"),
            (DashboardPanel::System, "System"),
            (DashboardPanel::Cells, "Cells"),
            (DashboardPanel::NeuralScheduler, "Scheduler"),
            (DashboardPanel::Mind, "Mind"),
            (DashboardPanel::Activities, "Activities"),
        ];
        
        let buttons: Vec<Element<'_, Message>> = panels
            .iter()
            .map(|(panel, label)| {
                let is_selected = self.selected_panel == *panel;
                button(text(*label))
                    .style(if is_selected {
                        iced::theme::Button::Primary
                    } else {
                        iced::theme::Button::Secondary
                    })
                    .on_press(Message::None) // Panel change handled internally
                    .into()
            })
            .collect();
        
        row(buttons)
            .spacing(5)
            .into()
    }
    
    fn render_overview(&self, theme: &KolibriTheme) -> Element<'_, Message> {
        let cpu_gauge = self.render_metric_card(
            "CPU",
            format!("{:.1}%", self.data.system.cpu_usage),
            self.data.system.cpu_usage / 100.0,
            theme.colors.primary,
        );
        
        let memory_gauge = self.render_metric_card(
            "Memory",
            format!("{:.1}%", self.data.memory_usage),
            self.data.system.memory_usage / 100.0,
            theme.colors.secondary,
        );
        
        let disk_gauge = self.render_metric_card(
            "Disk",
            format!("{:.1}%", self.data.system.disk_usage),
            self.data.system.disk_usage / 100.0,
            theme.colors.info,
        );
        
        // Cells status
        let cells_status = column![
            text("Cells Status").size(theme.typography.font_sizes.lg),
            Space::with_height(5),
            self.render_cell_status(&self.data.memory_cell),
            Space::with_height(5),
            self.render_cell_status(&self.data.processor_cell),
        ];
        
        // Suggestions
        let suggestions = if self.data.suggestions.is_empty() {
            container(text("No suggestions at this time").style(iced::theme::Text::Secondary))
                .padding(10)
        } else {
            column(
                self.data.suggestions
                    .iter()
                    .map(|s| self.render_suggestion(s, theme))
                    .collect()
            )
            .spacing(5)
            .into()
        };
        
        column![
            row![cpu_gauge, memory_gauge, disk_gauge]
                .spacing(15),
            Space::with_height(20),
            cells_status,
            Space::with_height(20),
            container(suggestions)
                .style(iced::theme::Container::Box)
                .padding(15)
                .width(Length::Fill),
        ]
        .spacing(10)
        .into()
    }
    
    fn render_metric_card(
        &self,
        label: &str,
        value: String,
        progress: f32,
        color: iced::Color,
    ) -> Element<'_, Message> {
        container(
            column![
                text(label).size(12).style(iced::theme::Text::Secondary),
                text(value).size(24),
                progress_bar(0.0..=1.0, progress)
                    .height(6)
                    .width(Length::Fill),
            ]
            .spacing(5)
            .align_items(iced::Alignment::Center),
        )
        .padding(15)
        .width(Length::FillPortion(1))
        .style(iced::theme::Container::Box)
        .into()
    }
    
    fn render_cell_status(&self, cell: &CellStatus) -> Element<'_, Message> {
        let state_color = match cell.health {
            HealthStatus::Healthy => iced::Color::from_rgb(0.3, 0.8, 0.5),
            HealthStatus::Warning => iced::Color::from_rgb(1.0, 0.75, 0.2),
            HealthStatus::Critical => iced::Color::from_rgb(1.0, 0.35, 0.35),
        };
        
        let state_text = match cell.state {
            CellState::Initializing => "Initializing",
            CellState::Active => "Active",
            CellState::Degraded => "Degraded",
            CellState::Healing => "Healing",
            CellState::Shutdown => "Shutdown",
        };
        
        row![
            text(&cell.name).width(Length::Fill),
            text(state_text).style(iced::theme::Text::Secondary),
            container(Space::with_width(10))
                .style(iced::theme::Container::Custom(Box::new(
                    move || iced::widget::container::Appearance {
                        background: Some(iced::Background::Color(state_color)),
                        border_radius: 5.0.into(),
                        ..Default::default()
                    }
                )))
                .width(10)
                .height(10),
        ]
        .spacing(10)
        .align_items(iced::Alignment::Center)
        .into()
    }
    
    fn render_suggestion(&self, suggestion: &Suggestion, theme: &KolibriTheme) -> Element<'_, Message> {
        let icon = match suggestion.suggestion_type {
            SuggestionType::Performance => "⚡",
            SuggestionType::Memory => "💾",
            SuggestionType::Security => "🔒",
            SuggestionType::Productivity => "📈",
            SuggestionType::Creative => "✨",
        };
        
        container(
            row![
                text(icon).size(20),
                column![
                    text(&suggestion.title).size(14),
                    text(&suggestion.description).size(12)
                        .style(iced::theme::Text::Secondary),
                ]
                .spacing(2),
            ]
            .spacing(10)
            .align_items(iced::Alignment::Center),
        )
        .padding(10)
        .style(iced::theme::Container::Box)
        .into()
    }
    
    fn render_system_panel(&self, theme: &KolibriTheme) -> Element<'_, Message> {
        column![
            text("System Metrics").size(theme.typography.font_sizes.xl),
            Space::with_height(10),
            self.render_metric_row("CPU Usage", format!("{:.1}%", self.data.system.cpu_usage)),
            self.render_metric_row("Memory Usage", format!("{:.1}%", self.data.system.memory_usage)),
            self.render_metric_row("Disk Usage", format!("{:.1}%", self.data.system.disk_usage)),
            self.render_metric_row("Network", format!("{:.1}%", self.data.system.network_usage)),
            self.render_metric_row("Processes", format!("{}", self.data.system.process_count)),
            self.render_metric_row("Threads", format!("{}", self.data.system.thread_count)),
            self.render_metric_row("Load Average", format!(
                "{:.2}, {:.2}, {:.2}",
                self.data.system.load_average[0],
                self.data.system.load_average[1],
                self.data.system.load_average[2]
            )),
        ]
        .spacing(8)
        .into()
    }
    
    fn render_metric_row(&self, label: &str, value: String) -> Element<'_, Message> {
        row![
            text(label).width(Length::Fill),
            text(value).style(iced::theme::Text::Secondary),
        ]
        .spacing(10)
        .into()
    }
    
    fn render_cells_panel(&self, theme: &KolibriTheme) -> Element<'_, Message> {
        column![
            text("Cells Status").size(theme.typography.font_sizes.xl),
            Space::with_height(10),
            text("Memory Cell").size(theme.typography.font_sizes.lg),
            self.render_cell_details(&self.data.memory_cell),
            Space::with_height(15),
            text("Processor Cell").size(theme.typography.font_sizes.lg),
            self.render_cell_details(&self.data.processor_cell),
        ]
        .spacing(8)
        .into()
    }
    
    fn render_cell_details(&self, cell: &CellStatus) -> Element<'_, Message> {
        column![
            self.render_metric_row("State", format!("{:?}", cell.state)),
            self.render_metric_row("Health", format!("{:?}", cell.health)),
            self.render_metric_row("Ops/sec", format!("{:.1}", cell.metrics.operations_per_second)),
            self.render_metric_row("Avg Latency", format!("{:.2}ms", cell.metrics.avg_latency_ms)),
            self.render_metric_row("Total Ops", format!("{}", cell.metrics.operations_total)),
            self.render_metric_row("Errors", format!("{}", cell.metrics.errors_total)),
        ]
        .spacing(4)
        .into()
    }
    
    fn render_neural_scheduler_panel(&self, theme: &KolibriTheme) -> Element<'_, Message> {
        column![
            text("Neural Scheduler").size(theme.typography.font_sizes.xl),
            Space::with_height(10),
            self.render_metric_row("Active", format!("{}", self.data.neural_scheduler.active)),
            self.render_metric_row("Current Decision", self.data.neural_scheduler.current_decision.clone()),
            self.render_metric_row("Confidence", format!("{:.1}%", self.data.neural_scheduler.confidence * 100.0)),
            self.render_metric_row("Decisions Today", format!("{}", self.data.neural_scheduler.decisions_today)),
            self.render_metric_row("Avg Confidence", format!("{:.1}%", self.data.neural_scheduler.avg_confidence * 100.0)),
        ]
        .spacing(8)
        .into()
    }
    
    fn render_mind_panel(&self, theme: &KolibriTheme) -> Element<'_, Message> {
        column![
            text("Unified Mind").size(theme.typography.font_sizes.xl),
            Space::with_height(10),
            self.render_metric_row("State", self.data.unified_mind.state.clone()),
            self.render_metric_row("LLM Provider", self.data.unified_mind.llm_provider.clone()),
            self.render_metric_row("Queries Processed", format!("{}", self.data.unified_mind.queries_processed)),
            self.render_metric_row("Uptime", self.data.unified_mind.uptime.clone()),
            self.render_metric_row("Context Items", format!("{}", self.data.unified_mind.context_items)),
        ]
        .spacing(8)
        .into()
    }
    
    fn render_activities_panel(&self, theme: &KolibriTheme) -> Element<'_, Message> {
        let activities: Vec<Element<'_, Message>> = self.data.activities
            .iter()
            .rev()
            .take(20)
            .map(|activity| {
                let time_str = activity.timestamp.elapsed().as_secs();
                let time_text = if time_str < 60 {
                    format!("{}s ago", time_str)
                } else if time_str < 3600 {
                    format!("{}m ago", time_str / 60)
                } else {
                    format!("{}h ago", time_str / 3600)
                };
                
                container(
                    row![
                        text(&activity.activity_type).width(Length::Fill),
                        text(&activity.description).width(Length::FillPortion(2)),
                        text(time_text).style(iced::theme::Text::Secondary),
                    ]
                    .spacing(10),
                )
                .padding(8)
                .style(iced::theme::Container::Box)
                .into()
            })
            .collect();
        
        column![
            text("Recent Activities").size(theme.typography.font_sizes.xl),
            Space::with_height(10),
            column(activities).spacing(5),
        ]
        .spacing(8)
        .into()
    }
}

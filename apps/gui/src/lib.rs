//! KolibriOS AI - Adaptive GUI Framework
//!
//! A revolutionary GUI framework that adapts to user context, Unified Mind directives,
//! and system state. Built on Iced for native performance and cross-platform support.

pub mod adaptive;
pub mod components;
pub mod theme;
pub mod layout;
pub mod animation;
pub mod mind_integration;
pub mod notifications;
pub mod dashboard;

pub use adaptive::*;
pub use components::*;
pub use theme::*;
pub use layout::*;
pub use animation::*;
pub use mind_integration::*;
pub use notifications::*;
pub use dashboard::*;

use iced::Application;
use iced::Settings;
use iced::window::Settings as WindowSettings;

/// KolibriGUI Configuration
#[derive(Debug, Clone)]
pub struct GuiConfig {
    /// Application title
    pub title: String,
    
    /// Initial window size
    pub window_size: (u32, u32),
    
    /// Enable adaptive UI
    pub adaptive_enabled: bool,
    
    /// Unified Mind endpoint
    pub mind_endpoint: String,
    
    /// Theme preference
    pub theme_preference: ThemePreference,
    
    /// Animation speed (0.0 - 1.0)
    pub animation_speed: f32,
    
    /// Enable context-aware suggestions
    pub context_suggestions: bool,
    
    /// Dashboard refresh rate in milliseconds
    pub dashboard_refresh_ms: u64,
}

impl Default for GuiConfig {
    fn default() -> Self {
        Self {
            title: "KolibriOS AI".to_string(),
            window_size: (1280, 720),
            adaptive_enabled: true,
            mind_endpoint: "http://localhost:50051".to_string(),
            theme_preference: ThemePreference::Auto,
            animation_speed: 0.7,
            context_suggestions: true,
            dashboard_refresh_ms: 1000,
        }
    }
}

/// Theme preference
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemePreference {
    Light,
    Dark,
    Auto,
}

/// Launch the KolibriOS AI GUI
pub fn launch(config: GuiConfig) -> iced::Result {
    let settings = Settings {
        window: WindowSettings {
            size: iced::Size::new(config.window_size.0 as f32, config.window_size.1 as f32),
            min_size: Some(iced::Size::new(800.0, 600.0)),
            title: config.title.clone(),
            resizable: true,
            decorations: true,
            transparent: false,
            ..Default::default()
        },
        ..Default::default()
    };
    
    KolibriApp::run(settings)
}

/// Main KolibriOS AI Application
#[derive(Debug)]
pub struct KolibriApp {
    /// Current view
    current_view: AppView,
    
    /// Adaptive state
    adaptive_state: AdaptiveState,
    
    /// Theme
    theme: KolibriTheme,
    
    /// Dashboard
    dashboard: Dashboard,
    
    /// Notifications
    notifications: NotificationManager,
    
    /// Mind client
    mind_client: MindClient,
    
    /// Config
    config: GuiConfig,
}

/// Application views
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppView {
    Dashboard,
    FileManager,
    CreativeAssistant,
    Settings,
    Diagnostics,
}

#[derive(Debug, Clone)]
pub enum Message {
    /// View changed
    ViewChanged(AppView),
    
    /// Dashboard update
    DashboardUpdate(dashboard::DashboardData),
    
    /// Notification received
    NotificationReceived(Notification),
    
    /// Theme changed
    ThemeChanged(KolibriTheme),
    
    /// Adaptive state changed
    AdaptiveStateChanged(AdaptiveState),
    
    /// Mind directive received
    MindDirective(MindDirective),
    
    /// Tick for animations
    Tick,
    
    /// File manager event
    FileManagerEvent(file_manager::Message),
    
    /// Creative assistant event
    CreativeAssistantEvent(creative_assistant::Message),
    
    /// No operation
    None,
}

impl Application for KolibriApp {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = iced::Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, iced::Command<Message>) {
        let config = GuiConfig::default();
        
        (
            Self {
                current_view: AppView::Dashboard,
                adaptive_state: AdaptiveState::default(),
                theme: KolibriTheme::default(),
                dashboard: Dashboard::new(),
                notifications: NotificationManager::new(),
                mind_client: MindClient::new(&config.mind_endpoint),
                config,
            },
            iced::Command::none(),
        )
    }

    fn title(&self) -> String {
        self.config.title.clone()
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Message> {
        match message {
            Message::ViewChanged(view) => {
                self.current_view = view;
                iced::Command::none()
            }
            
            Message::DashboardUpdate(data) => {
                self.dashboard.update(data);
                iced::Command::none()
            }
            
            Message::NotificationReceived(notification) => {
                self.notifications.add(notification);
                iced::Command::none()
            }
            
            Message::ThemeChanged(theme) => {
                self.theme = theme;
                iced::Command::none()
            }
            
            Message::AdaptiveStateChanged(state) => {
                self.adaptive_state = state;
                // Apply adaptive changes
                self.apply_adaptive_changes();
                iced::Command::none()
            }
            
            Message::MindDirective(directive) => {
                self.handle_mind_directive(directive)
            }
            
            Message::Tick => {
                // Update animations
                self.dashboard.tick();
                iced::Command::none()
            }
            
            Message::FileManagerEvent(msg) => {
                // Handle file manager events
                iced::Command::none()
            }
            
            Message::CreativeAssistantEvent(msg) => {
                // Handle creative assistant events
                iced::Command::none()
            }
            
            Message::None => iced::Command::none(),
        }
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        let sidebar = self.render_sidebar();
        let main_content = self.render_main_content();
        let notifications = self.notifications.view();
        
        let content = iced::widget::row![sidebar, main_content]
            .spacing(0)
            .height(iced::Length::Fill);
        
        iced::widget::column![content, notifications]
            .height(iced::Length::Fill)
            .into()
    }

    fn theme(&self) -> Self::Theme {
        self.theme.to_iced_theme()
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        // Subscribe to dashboard updates and mind directives
        iced::Subscription::batch(vec![
            self.dashboard.subscription(),
            self.mind_client.subscription(),
        ])
    }
}

impl KolibriApp {
    fn render_sidebar(&self) -> iced::Element<'_, Message> {
        use iced::widget::{button, column, container, text, Space};
        
        let menu_items = vec![
            (AppView::Dashboard, "Dashboard", "📊"),
            (AppView::FileManager, "Files", "📁"),
            (AppView::CreativeAssistant, "Creative", "✨"),
            (AppView::Diagnostics, "Diagnostics", "🔧"),
            (AppView::Settings, "Settings", "⚙️"),
        ];
        
        let menu: Vec<iced::Element<'_, Message>> = menu_items
            .into_iter()
            .map(|(view, label, icon)| {
                let is_active = self.current_view == view;
                let btn = button(
                    iced::widget::row![]
                        .push(text(icon).size(18))
                        .push(text(label).size(14))
                        .spacing(10)
                        .align_items(iced::Alignment::Center)
                )
                .width(iced::Length::Fill)
                .style(if is_active {
                    iced::theme::Button::Primary
                } else {
                    iced::theme::Button::Text
                })
                .on_press(Message::ViewChanged(view));
                
                container(btn)
                    .width(iced::Length::Fill)
                    .padding(5)
                    .into()
            })
            .collect();
        
        container(
            column![
                text("KolibriOS AI").size(18),
                Space::with_height(20),
                column(menu)
            ]
            .spacing(5)
        )
        .width(iced::Length::Fixed(180.0))
        .height(iced::Length::Fill)
        .padding(15)
        .style(iced::theme::Container::Box)
        .into()
    }
    
    fn render_main_content(&self) -> iced::Element<'_, Message> {
        match self.current_view {
            AppView::Dashboard => self.dashboard.view(),
            AppView::FileManager => {
                // File manager view
                iced::widget::text("File Manager").into()
            }
            AppView::CreativeAssistant => {
                // Creative assistant view
                iced::widget::text("Creative Assistant").into()
            }
            AppView::Diagnostics => {
                // Diagnostics view
                iced::widget::text("Diagnostics").into()
            }
            AppView::Settings => {
                // Settings view
                iced::widget::text("Settings").into()
            }
        }
    }
    
    fn apply_adaptive_changes(&mut self) {
        // Apply adaptive changes based on state
        if self.adaptive_state.focus_mode {
            // Enable focus mode - minimize distractions
            self.theme = KolibriTheme::focus_mode();
        }
        
        if self.adaptive_state.performance_mode {
            // Reduce animations for performance
            self.config.animation_speed = 0.3;
        }
    }
    
    fn handle_mind_directive(&mut self, directive: MindDirective) -> iced::Command<Message> {
        match directive {
            MindDirective::SwitchView(view_name) => {
                if let Ok(view) = self.parse_view_name(&view_name) {
                    self.current_view = view;
                }
                iced::Command::none()
            }
            
            MindDirective::ShowNotification(notification) => {
                self.notifications.add(notification);
                iced::Command::none()
            }
            
            MindDirective::UpdateTheme(theme_name) => {
                self.theme = KolibriTheme::from_name(&theme_name);
                iced::Command::none()
            }
            
            MindDirective::OptimizeStorage => {
                // Trigger storage optimization
                self.notifications.add(Notification::info(
                    "Storage Optimization",
                    "Analyizing storage and optimizing..."
                ));
                iced::Command::none()
            }
            
            MindDirective::SuggestFiles(files) => {
                // Show file suggestions
                let suggestion = format!("Suggested files: {}", files.join(", "));
                self.notifications.add(Notification::info(
                    "File Suggestions",
                    suggestion
                ));
                iced::Command::none()
            }
            
            MindDirective::CreativePrompt(prompt) => {
                // Switch to creative assistant with prompt
                self.current_view = AppView::CreativeAssistant;
                iced::Command::none()
            }
            
            MindDirective::None => iced::Command::none(),
        }
    }
    
    fn parse_view_name(&self, name: &str) -> Result<AppView, ()> {
        match name.to_lowercase().as_str() {
            "dashboard" => Ok(AppView::Dashboard),
            "files" | "filemanager" => Ok(AppView::FileManager),
            "creative" | "assistant" => Ok(AppView::CreativeAssistant),
            "diagnostics" => Ok(AppView::Diagnostics),
            "settings" => Ok(AppView::Settings),
            _ => Err(()),
        }
    }
}

/// Placeholder modules until we create full implementations
pub mod file_manager {
    #[derive(Debug, Clone)]
    pub enum Message {}
}

pub mod creative_assistant {
    #[derive(Debug, Clone)]
    pub enum Message {}
}

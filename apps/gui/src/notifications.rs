//! Adaptive Notification System
//!
//! A smart notification system that prioritizes and displays notifications
//! based on context and user preferences.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use iced::widget::{button, column, container, row, text, Space};
use iced::{Element, Length};

use super::theme::KolibriTheme;

/// Notification Manager
#[derive(Debug)]
pub struct NotificationManager {
    /// Active notifications
    notifications: VecDeque<Notification>,
    
    /// Notification history
    history: VecDeque<Notification>,
    
    /// Maximum visible notifications
    max_visible: usize,
    
    /// Maximum history size
    max_history: usize,
    
    /// Default duration
    default_duration: Duration,
    
    /// Settings
    settings: NotificationSettings,
}

/// Notification
#[derive(Debug, Clone)]
pub struct Notification {
    pub id: u64,
    pub notification_type: NotificationType,
    pub title: String,
    pub message: String,
    pub severity: NotificationSeverity,
    pub source: String,
    pub created_at: Instant,
    pub expires_at: Option<Instant>,
    pub actions: Vec<NotificationAction>,
    pub read: bool,
    pub dismissed: bool,
    pub priority: u8,
}

/// Notification type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationType {
    /// System notification
    System,
    /// Cell notification
    Cell,
    /// Unified Mind notification
    Mind,
    /// Application notification
    Application,
    /// User notification
    User,
    /// Alert notification
    Alert,
}

/// Notification severity
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationSeverity {
    Info,
    Success,
    Warning,
    Error,
    Critical,
}

/// Notification action
#[derive(Debug, Clone)]
pub struct NotificationAction {
    pub label: String,
    pub action_type: NotificationActionType,
}

#[derive(Debug, Clone)]
pub enum NotificationActionType {
    Dismiss,
    View,
    Action(String),
    OpenUrl(String),
    OpenFile(String),
    OpenApp(String),
}

/// Notification settings
#[derive(Debug, Clone)]
pub struct NotificationSettings {
    pub enabled: bool,
    pub sound_enabled: bool,
    pub do_not_disturb: bool,
    pub do_not_disturb_schedule: Option<(u8, u8)>, // Start hour, end hour
    pub group_similar: bool,
    pub show_previews: bool,
    pub position: NotificationPosition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationPosition {
    TopRight,
    TopLeft,
    BottomRight,
    BottomLeft,
    Center,
}

impl Default for NotificationSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            sound_enabled: true,
            do_not_disturb: false,
            do_not_disturb_schedule: Some((22, 7)), // 10 PM to 7 AM
            group_similar: true,
            show_previews: true,
            position: NotificationPosition::BottomRight,
        }
    }
}

impl Default for NotificationManager {
    fn default() -> Self {
        Self::new()
    }
}

impl NotificationManager {
    pub fn new() -> Self {
        Self {
            notifications: VecDeque::with_capacity(50),
            history: VecDeque::with_capacity(200),
            max_visible: 5,
            max_history: 200,
            default_duration: Duration::from_secs(10),
            settings: NotificationSettings::default(),
        }
    }
    
    /// Add a new notification
    pub fn add(&mut self, notification: Notification) {
        if !self.settings.enabled {
            return;
        }
        
        // Check do not disturb
        if self.should_suppress() {
            // Still add to history but don't show
            self.history.push_back(notification);
            return;
        }
        
        // Group similar notifications
        if self.settings.group_similar {
            if let Some(existing) = self.notifications.iter_mut().find(|n| {
                n.title == notification.title && n.source == notification.source
            }) {
                existing.message = format!("{} (and similar)", existing.message);
                return;
            }
        }
        
        // Remove oldest if at capacity
        while self.notifications.len() >= self.max_visible {
            if let Some(old) = self.notifications.pop_front() {
                self.history.push_back(old);
            }
        }
        
        // Trim history
        while self.history.len() >= self.max_history {
            self.history.pop_front();
        }
        
        self.notifications.push_back(notification);
    }
    
    /// Create an info notification
    pub fn info(title: impl Into<String>, message: impl Into<String>) -> Notification {
        Notification {
            id: rand::random(),
            notification_type: NotificationType::System,
            title: title.into(),
            message: message.into(),
            severity: NotificationSeverity::Info,
            source: "System".to_string(),
            created_at: Instant::now(),
            expires_at: Some(Instant::now() + Duration::from_secs(10)),
            actions: vec![NotificationAction::dismiss()],
            read: false,
            dismissed: false,
            priority: 0,
        }
    }
    
    /// Create a success notification
    pub fn success(title: impl Into<String>, message: impl Into<String>) -> Notification {
        Notification {
            id: rand::random(),
            notification_type: NotificationType::System,
            title: title.into(),
            message: message.into(),
            severity: NotificationSeverity::Success,
            source: "System".to_string(),
            created_at: Instant::now(),
            expires_at: Some(Instant::now() + Duration::from_secs(8)),
            actions: vec![NotificationAction::dismiss()],
            read: false,
            dismissed: false,
            priority: 0,
        }
    }
    
    /// Create a warning notification
    pub fn warning(title: impl Into<String>, message: impl Into<String>) -> Notification {
        Notification {
            id: rand::random(),
            notification_type: NotificationType::Alert,
            title: title.into(),
            message: message.into(),
            severity: NotificationSeverity::Warning,
            source: "System".to_string(),
            created_at: Instant::now(),
            expires_at: Some(Instant::now() + Duration::from_secs(15)),
            actions: vec![NotificationAction::dismiss()],
            read: false,
            dismissed: false,
            priority: 1,
        }
    }
    
    /// Create an error notification
    pub fn error(title: impl Into<String>, message: impl Into<String>) -> Notification {
        Notification {
            id: rand::random(),
            notification_type: NotificationType::Alert,
            title: title.into(),
            message: message.into(),
            severity: NotificationSeverity::Error,
            source: "System".to_string(),
            created_at: Instant::now(),
            expires_at: None, // Errors don't auto-dismiss
            actions: vec![NotificationAction::dismiss()],
            read: false,
            dismissed: false,
            priority: 2,
        }
    }
    
    /// Dismiss a notification
    pub fn dismiss(&mut self, id: u64) {
        if let Some(notification) = self.notifications.iter_mut().find(|n| n.id == id) {
            notification.dismissed = true;
        }
    }
    
    /// Dismiss all notifications
    pub fn dismiss_all(&mut self) {
        for notification in self.notifications.iter_mut() {
            notification.dismissed = true;
        }
    }
    
    /// Tick - update expired notifications
    pub fn tick(&mut self) {
        let now = Instant::now();
        
        // Remove expired notifications
        self.notifications.retain(|n| {
            if let Some(expires) = n.expires_at {
                if now >= expires || n.dismissed {
                    return false;
                }
            }
            !n.dismissed
        });
    }
    
    /// Check if should suppress notifications (do not disturb)
    fn should_suppress(&self) -> bool {
        if self.settings.do_not_disturb {
            return true;
        }
        
        if let Some((start, end)) = self.settings.do_not_disturb_schedule {
            let hour = chrono::Local::now().hour() as u8;
            if start > end {
                // Overnight schedule (e.g., 22:00 - 07:00)
                if hour >= start || hour < end {
                    return true;
                }
            } else {
                // Same-day schedule
                if hour >= start && hour < end {
                    return true;
                }
            }
        }
        
        false
    }
    
    /// Get unread count
    pub fn unread_count(&self) -> usize {
        self.notifications.iter().filter(|n| !n.read).count()
    }
    
    /// Get notification count
    pub fn count(&self) -> usize {
        self.notifications.len()
    }
    
    /// Render notifications
    pub fn view(&self) -> Element<'_, super::Message> {
        let theme = KolibriTheme::default();
        
        if self.notifications.is_empty() {
            return Space::with_height(0).into();
        }
        
        let notification_views: Vec<Element<'_, super::Message>> = self.notifications
            .iter()
            .map(|n| self.render_notification(n, &theme))
            .collect();
        
        container(
            column(notification_views)
                .spacing(8)
        )
        .padding(10)
        .width(Length::Fill)
        .into()
    }
    
    fn render_notification(&self, notification: &Notification, theme: &KolibriTheme) -> Element<'_, super::Message> {
        let severity_color = match notification.severity {
            NotificationSeverity::Info => theme.colors.info,
            NotificationSeverity::Success => theme.colors.success,
            NotificationSeverity::Warning => theme.colors.warning,
            NotificationSeverity::Error | NotificationSeverity::Critical => theme.colors.error,
        };
        
        let icon = match notification.severity {
            NotificationSeverity::Info => "ℹ️",
            NotificationSeverity::Success => "✅",
            NotificationSeverity::Warning => "⚠️",
            NotificationSeverity::Error => "❌",
            NotificationSeverity::Critical => "🚨",
        };
        
        container(
            row![
                // Severity indicator
                container(Space::with_width(4))
                    .style(iced::theme::Container::Custom(Box::new(move || {
                        iced::widget::container::Appearance {
                            background: Some(iced::Background::Color(severity_color)),
                            ..Default::default()
                        }
                    })))
                    .width(4)
                    .height(Length::Fill),
                
                // Content
                column![
                    row![
                        text(icon).size(14),
                        text(&notification.title).size(14),
                    ]
                    .spacing(8),
                    text(&notification.message).size(12)
                        .style(iced::theme::Text::Secondary),
                ]
                .spacing(4)
                .padding(10),
                
                Space::with_width(Length::Fill),
                
                // Dismiss button
                button(text("×").size(16))
                    .style(iced::theme::Button::Text)
                    .on_press(super::Message::None),
            ]
            .spacing(0)
            .align_items(iced::Alignment::Center),
        )
        .style(iced::theme::Container::Box)
        .width(Length::Fill)
        .max_width(400)
        .into()
    }
}

impl NotificationAction {
    pub fn dismiss() -> Self {
        Self {
            label: "Dismiss".to_string(),
            action_type: NotificationActionType::Dismiss,
        }
    }
    
    pub fn view() -> Self {
        Self {
            label: "View".to_string(),
            action_type: NotificationActionType::View,
        }
    }
    
    pub fn action(label: impl Into<String>, action: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            action_type: NotificationActionType::Action(action.into()),
        }
    }
    
    pub fn open_file(path: impl Into<String>) -> Self {
        Self {
            label: "Open File".to_string(),
            action_type: NotificationActionType::OpenFile(path.into()),
        }
    }
}

//! Core UI Components
//!
//! Reusable adaptive UI components for KolibriOS AI.

pub mod buttons;
pub mod cards;
pub mod inputs;
pub mod lists;
pub mod panels;
pub mod progress;
pub mod sidebar;

pub use buttons::*;
pub use cards::*;
pub use inputs::*;
pub use lists::*;
pub use panels::*;
pub use progress::*;
pub use sidebar::*;

use iced::widget::{button, container, text, row, column, Space};
use iced::{Element, Length};

use crate::theme::KolibriTheme;

/// Adaptive button component
pub fn adaptive_button<'a, Message: 'a + Clone>(
    label: &str,
    style: ButtonStyle,
) -> Element<'a, Message> {
    let theme = KolibriTheme::default();
    
    let btn = button(text(label))
        .padding(theme.spacing.sm);
    
    match style {
        ButtonStyle::Primary => btn.style(iced::theme::Button::Primary),
        ButtonStyle::Secondary => btn.style(iced::theme::Button::Secondary),
        ButtonStyle::Success => btn.style(iced::theme::Button::Success),
        ButtonStyle::Danger => btn.style(iced::theme::Button::Danger),
        ButtonStyle::Text => btn.style(iced::theme::Button::Text),
    }
    .into()
}

/// Button style variants
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonStyle {
    Primary,
    Secondary,
    Success,
    Danger,
    Text,
}

/// Adaptive card component
pub fn adaptive_card<'a, Message: 'a + Clone>(
    title: &str,
    content: Element<'a, Message>,
) -> Element<'a, Message> {
    let theme = KolibriTheme::default();
    
    container(
        column![
            text(title).size(theme.typography.font_sizes.lg),
            Space::with_height(theme.spacing.sm),
            content,
        ]
        .spacing(theme.spacing.xs)
    )
    .padding(theme.spacing.md)
    .style(iced::theme::Container::Box)
    .width(Length::Fill)
    .into()
}

/// Status badge component
pub fn status_badge<'a>(status: StatusType, label: &str) -> Element<'a, crate::Message> {
    let theme = KolibriTheme::default();
    let color = match status {
        StatusType::Success => theme.colors.success,
        StatusType::Warning => theme.colors.warning,
        StatusType::Error => theme.colors.error,
        StatusType::Info => theme.colors.info,
        StatusType::Neutral => theme.colors.text_secondary,
    };
    
    container(text(label).size(theme.typography.font_sizes.xs))
        .padding([theme.spacing.xxs, theme.spacing.xs])
        .style(iced::theme::Container::Custom(Box::new(move || {
            iced::widget::container::Appearance {
                background: Some(iced::Background::Color(iced::Color::from_rgba(
                    color.r, color.g, color.b, 0.2
                ))),
                text_color: Some(color),
                border_radius: theme.radius.full.into(),
                ..Default::default()
            }
        })))
        .into()
}

/// Status type for badges
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusType {
    Success,
    Warning,
    Error,
    Info,
    Neutral,
}

/// Loading spinner component
pub fn loading_spinner<'a>() -> Element<'a, crate::Message> {
    text("Loading...").style(iced::theme::Text::Secondary).into()
}

/// Empty state component
pub fn empty_state<'a, Message: 'a + Clone>(
    icon: &str,
    title: &str,
    description: &str,
) -> Element<'a, Message> {
    let theme = KolibriTheme::default();
    
    container(
        column![
            text(icon).size(48),
            Space::with_height(theme.spacing.md),
            text(title).size(theme.typography.font_sizes.lg),
            text(description)
                .size(theme.typography.font_sizes.sm)
                .style(iced::theme::Text::Secondary),
        ]
        .spacing(theme.spacing.xs)
        .align_items(iced::Alignment::Center)
    )
    .padding(theme.spacing.xxl)
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x()
    .center_y()
    .into()
}

/// Section header component
pub fn section_header<'a>(title: &str, subtitle: Option<&str>) -> Element<'a, crate::Message> {
    let theme = KolibriTheme::default();
    
    let title_element = text(title).size(theme.typography.font_sizes.xl);
    
    match subtitle {
        Some(sub) => column![
            title_element,
            text(sub)
                .size(theme.typography.font_sizes.sm)
                .style(iced::theme::Text::Secondary),
        ]
        .spacing(theme.spacing.xxs)
        .into(),
        None => title_element.into(),
    }
}

/// Breadcrumb navigation
pub fn breadcrumb<'a>(items: &[&str]) -> Element<'a, crate::Message> {
    let theme = KolibriTheme::default();
    
    let elements: Vec<Element<'a, crate::Message>> = items
        .iter()
        .enumerate()
        .flat_map(|(i, item)| {
            let is_last = i == items.len() - 1;
            let text_elem = if is_last {
                text(*item).style(iced::theme::Text::Default)
            } else {
                text(*item).style(iced::theme::Text::Secondary)
            };
            
            if is_last {
                vec![text_elem.into()]
            } else {
                vec![
                    text_elem.into(),
                    text("/").style(iced::theme::Text::Secondary).into(),
                ]
            }
        })
        .collect();
    
    row(elements)
        .spacing(theme.spacing.xs)
        .into()
}

/// Search bar component
pub fn search_bar<'a, Message: 'a + Clone>(placeholder: &str) -> Element<'a, Message> {
    let theme = KolibriTheme::default();
    
    container(
        row![
            text("🔍").size(16),
            iced::widget::text_input("", placeholder)
                .padding(theme.spacing.sm)
                .width(Length::Fill),
        ]
        .spacing(theme.spacing.sm)
        .align_items(iced::Alignment::Center)
    )
    .padding([theme.spacing.xs, theme.spacing.sm])
    .style(iced::theme::Container::Box)
    .width(Length::Fill)
    .into()
}

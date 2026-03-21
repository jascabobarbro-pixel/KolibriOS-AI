//! KolibriOS AI GUI - Main Entry Point
//!
//! Launch the adaptive GUI with living applications.

mod lib;

use kolibri_gui::{GuiConfig, launch, AppView, Message};

fn main() -> iced::Result {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    let config = GuiConfig {
        title: "KolibriOS AI - Adaptive Interface".to_string(),
        window_size: (1280, 800),
        adaptive_enabled: true,
        mind_endpoint: "http://localhost:50051".to_string(),
        theme_preference: kolibri_gui::ThemePreference::Auto,
        animation_speed: 0.7,
        context_suggestions: true,
        dashboard_refresh_ms: 1000,
    };
    
    tracing::info!("Starting KolibriOS AI GUI");
    tracing::info!("Mind endpoint: {}", config.mind_endpoint);
    
    launch(config)
}

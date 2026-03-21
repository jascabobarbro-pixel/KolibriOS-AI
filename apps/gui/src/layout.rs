//! Adaptive Layout System
//!
//! Dynamic layout management that responds to context and device constraints.

use std::collections::HashMap;

/// Layout manager for adaptive positioning
#[derive(Debug, Clone)]
pub struct LayoutManager {
    /// Current layout configuration
    config: LayoutConfig,
    
    /// Container positions
    containers: HashMap<String, ContainerLayout>,
    
    /// Grid configuration
    grid: GridConfig,
    
    /// Responsive breakpoints
    breakpoints: Breakpoints,
}

/// Layout configuration
#[derive(Debug, Clone)]
pub struct LayoutConfig {
    /// Layout mode
    pub mode: LayoutMode,
    
    /// Main sidebar position
    pub sidebar_position: SidebarPosition,
    
    /// Content width mode
    pub content_width: ContentWidth,
    
    /// Gap between elements
    pub gap: f32,
    
    /// Padding around edges
    pub padding: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutMode {
    /// Desktop layout with sidebar
    Desktop,
    /// Tablet layout with collapsible sidebar
    Tablet,
    /// Mobile layout with drawer
    Mobile,
    /// Focus mode (minimal UI)
    Focus,
    /// Presentation mode (full screen content)
    Presentation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidebarPosition {
    Left,
    Right,
    Hidden,
    Overlay,
}

#[derive(Debug, Clone, Copy)]
pub enum ContentWidth {
    Full,
    Centered { max_width: f32 },
    Responsive { min: f32, max: f32 },
}

/// Container layout information
#[derive(Debug, Clone)]
pub struct ContainerLayout {
    pub id: String,
    pub position: Position,
    pub size: Size,
    pub visible: bool,
    pub z_index: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

/// Grid configuration
#[derive(Debug, Clone)]
pub struct GridConfig {
    pub columns: u32,
    pub rows: u32,
    pub column_gap: f32,
    pub row_gap: f32,
    pub auto_flow: GridAutoFlow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridAutoFlow {
    Row,
    Column,
    Dense,
    RowDense,
    ColumnDense,
}

/// Responsive breakpoints
#[derive(Debug, Clone, Copy)]
pub struct Breakpoints {
    pub mobile: f32,
    pub tablet: f32,
    pub desktop: f32,
    pub wide: f32,
}

impl Default for Breakpoints {
    fn default() -> Self {
        Self {
            mobile: 480.0,
            tablet: 768.0,
            desktop: 1024.0,
            wide: 1440.0,
        }
    }
}

impl Default for LayoutManager {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutManager {
    pub fn new() -> Self {
        Self {
            config: LayoutConfig::default(),
            containers: HashMap::new(),
            grid: GridConfig::default(),
            breakpoints: Breakpoints::default(),
        }
    }
    
    /// Update layout based on window size
    pub fn update_for_size(&mut self, width: f32, height: f32) {
        self.config.mode = if width < self.breakpoints.mobile {
            LayoutMode::Mobile
        } else if width < self.breakpoints.tablet {
            LayoutMode::Tablet
        } else {
            LayoutMode::Desktop
        };
        
        // Adjust sidebar
        self.config.sidebar_position = match self.config.mode {
            LayoutMode::Mobile => SidebarPosition::Overlay,
            LayoutMode::Tablet => SidebarPosition::Left,
            LayoutMode::Desktop => SidebarPosition::Left,
            LayoutMode::Focus => SidebarPosition::Hidden,
            LayoutMode::Presentation => SidebarPosition::Hidden,
        };
        
        // Adjust grid
        self.grid.columns = match self.config.mode {
            LayoutMode::Mobile => 1,
            LayoutMode::Tablet => 2,
            LayoutMode::Desktop => 3,
            LayoutMode::Focus => 1,
            LayoutMode::Presentation => 1,
        };
    }
    
    /// Add a container to the layout
    pub fn add_container(&mut self, id: &str, layout: ContainerLayout) {
        self.containers.insert(id.to_string(), layout);
    }
    
    /// Get container layout
    pub fn get_container(&self, id: &str) -> Option<&ContainerLayout> {
        self.containers.get(id)
    }
    
    /// Calculate grid position for item
    pub fn grid_position(&self, index: usize) -> (u32, u32) {
        let col = (index as u32) % self.grid.columns;
        let row = (index as u32) / self.grid.columns;
        (col, row)
    }
    
    /// Get current layout mode
    pub fn mode(&self) -> LayoutMode {
        self.config.mode
    }
    
    /// Check if should show sidebar
    pub fn show_sidebar(&self) -> bool {
        self.config.sidebar_position != SidebarPosition::Hidden
    }
    
    /// Get sidebar width
    pub fn sidebar_width(&self) -> f32 {
        match self.config.mode {
            LayoutMode::Mobile => 280.0,
            LayoutMode::Tablet => 220.0,
            LayoutMode::Desktop => 240.0,
            LayoutMode::Focus => 0.0,
            LayoutMode::Presentation => 0.0,
        }
    }
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            mode: LayoutMode::Desktop,
            sidebar_position: SidebarPosition::Left,
            content_width: ContentWidth::Responsive {
                min: 320.0,
                max: 1200.0,
            },
            gap: 16.0,
            padding: 16.0,
        }
    }
}

impl Default for GridConfig {
    fn default() -> Self {
        Self {
            columns: 3,
            rows: 0, // Auto
            column_gap: 16.0,
            row_gap: 16.0,
            auto_flow: GridAutoFlow::Row,
        }
    }
}

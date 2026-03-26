//! Viewport - Canvas viewport and zoom/pan management
//!
//! Handles canvas coordinate transformations, zoom levels, and viewport state.

use serde::{Serialize, Deserialize};

/// Canvas viewport state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Viewport {
    /// Center X in canvas coordinates
    pub center_x: f64,
    /// Center Y in canvas coordinates
    pub center_y: f64,
    /// Zoom level (1.0 = 100%)
    pub zoom: f64,
    /// Viewport width in screen pixels
    pub width: f64,
    /// Viewport height in screen pixels
    pub height: f64,
    /// Minimum zoom level
    pub min_zoom: f64,
    /// Maximum zoom level
    pub max_zoom: f64,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            center_x: 0.0,
            center_y: 0.0,
            zoom: 1.0,
            width: 800.0,
            height: 600.0,
            min_zoom: 0.1,
            max_zoom: 5.0,
        }
    }
}

impl Viewport {
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            width,
            height,
            ..Default::default()
        }
    }

    /// Set zoom level (clamped to min/max)
    pub fn set_zoom(&mut self, zoom: f64) {
        self.zoom = zoom.clamp(self.min_zoom, self.max_zoom);
    }

    /// Zoom in by factor
    pub fn zoom_in(&mut self, factor: f64) {
        self.set_zoom(self.zoom * factor);
    }

    /// Zoom out by factor
    pub fn zoom_out(&mut self, factor: f64) {
        self.set_zoom(self.zoom / factor);
    }

    /// Zoom to fit bounds
    pub fn fit_bounds(&mut self, min_x: f64, min_y: f64, max_x: f64, max_y: f64, padding: f64) {
        let content_width = (max_x - min_x) + padding * 2.0;
        let content_height = (max_y - min_y) + padding * 2.0;

        if content_width > 0.0 && content_height > 0.0 {
            let zoom_x = self.width / content_width;
            let zoom_y = self.height / content_height;
            self.set_zoom(zoom_x.min(zoom_y));
            
            self.center_x = (min_x + max_x) / 2.0;
            self.center_y = (min_y + max_y) / 2.0;
        }
    }

    /// Pan by screen pixels
    pub fn pan(&mut self, dx: f64, dy: f64) {
        self.center_x -= dx / self.zoom;
        self.center_y -= dy / self.zoom;
    }

    /// Convert screen coordinates to canvas coordinates
    pub fn screen_to_canvas(&self, screen_x: f64, screen_y: f64) -> (f64, f64) {
        let canvas_x = (screen_x - self.width / 2.0) / self.zoom + self.center_x;
        let canvas_y = (screen_y - self.height / 2.0) / self.zoom + self.center_y;
        (canvas_x, canvas_y)
    }

    /// Convert canvas coordinates to screen coordinates
    pub fn canvas_to_screen(&self, canvas_x: f64, canvas_y: f64) -> (f64, f64) {
        let screen_x = (canvas_x - self.center_x) * self.zoom + self.width / 2.0;
        let screen_y = (canvas_y - self.center_y) * self.zoom + self.height / 2.0;
        (screen_x, screen_y)
    }

    /// Get visible bounds in canvas coordinates
    pub fn visible_bounds(&self) -> (f64, f64, f64, f64) {
        let half_w = (self.width / 2.0) / self.zoom;
        let half_h = (self.height / 2.0) / self.zoom;
        (
            self.center_x - half_w,
            self.center_y - half_h,
            self.center_x + half_w,
            self.center_y + half_h,
        )
    }

    /// Check if a point is visible
    pub fn is_visible(&self, x: f64, y: f64) -> bool {
        let (min_x, min_y, max_x, max_y) = self.visible_bounds();
        x >= min_x && x <= max_x && y >= min_y && y <= max_y
    }

    /// Check if a rectangle is visible (any part)
    pub fn is_rect_visible(&self, x: f64, y: f64, w: f64, h: f64) -> bool {
        let (vmin_x, vmin_y, vmax_x, vmax_y) = self.visible_bounds();
        !(x + w < vmin_x || x > vmax_x || y + h < vmin_y || y > vmax_y)
    }

    /// Reset to default view
    pub fn reset(&mut self) {
        self.center_x = 0.0;
        self.center_y = 0.0;
        self.zoom = 1.0;
    }

    /// Get transform matrix for CSS/SVG
    pub fn get_transform(&self) -> ViewportTransform {
        ViewportTransform {
            translate_x: self.width / 2.0 - self.center_x * self.zoom,
            translate_y: self.height / 2.0 - self.center_y * self.zoom,
            scale: self.zoom,
        }
    }
}

/// Transform data for rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewportTransform {
    pub translate_x: f64,
    pub translate_y: f64,
    pub scale: f64,
}

impl ViewportTransform {
    /// Get CSS transform string
    pub fn to_css(&self) -> String {
        format!(
            "translate({}px, {}px) scale({})",
            self.translate_x, self.translate_y, self.scale
        )
    }

    /// Get SVG transform string
    pub fn to_svg(&self) -> String {
        format!(
            "translate({}, {}) scale({})",
            self.translate_x, self.translate_y, self.scale
        )
    }
}

/// Minimap data for navigation overview
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinimapData {
    /// Bounding box of all content
    pub content_bounds: (f64, f64, f64, f64),
    /// Current viewport rectangle (in content coordinates)
    pub viewport_rect: (f64, f64, f64, f64),
    /// Simplified node positions for rendering
    pub node_positions: Vec<MinimapNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinimapNode {
    pub id: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub selected: bool,
    pub node_type: String,
}

impl MinimapNode {
    pub fn new(id: String, x: f64, y: f64, width: f64, height: f64, selected: bool, node_type: String) -> Self {
        Self { id, x, y, width, height, selected, node_type }
    }
}

impl MinimapData {
    pub fn new() -> Self {
        Self {
            content_bounds: (0.0, 0.0, 0.0, 0.0),
            viewport_rect: (0.0, 0.0, 0.0, 0.0),
            node_positions: Vec::new(),
        }
    }
    
    /// Convert a minimap click position to canvas coordinates
    /// 
    /// minimap_x, minimap_y: click position in minimap widget (0 to minimap_width, 0 to minimap_height)
    /// minimap_width, minimap_height: size of minimap widget
    pub fn click_to_canvas(&self, minimap_x: f64, minimap_y: f64, minimap_width: f64, minimap_height: f64) -> (f64, f64) {
        let (min_x, min_y, max_x, max_y) = self.content_bounds;
        let content_width = max_x - min_x;
        let content_height = max_y - min_y;
        
        if content_width <= 0.0 || content_height <= 0.0 {
            return (0.0, 0.0);
        }
        
        // Map minimap coordinates to canvas coordinates
        let canvas_x = min_x + (minimap_x / minimap_width) * content_width;
        let canvas_y = min_y + (minimap_y / minimap_height) * content_height;
        
        (canvas_x, canvas_y)
    }
}

impl Default for MinimapData {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_viewport_coords() {
        let mut vp = Viewport::new(800.0, 600.0);
        vp.center_x = 100.0;
        vp.center_y = 100.0;
        vp.zoom = 1.0;

        let (cx, cy) = vp.screen_to_canvas(400.0, 300.0);
        assert!((cx - 100.0).abs() < 0.1);
        assert!((cy - 100.0).abs() < 0.1);
    }

    #[test]
    fn test_zoom_clamp() {
        let mut vp = Viewport::default();
        vp.set_zoom(10.0);
        assert_eq!(vp.zoom, vp.max_zoom);

        vp.set_zoom(0.01);
        assert_eq!(vp.zoom, vp.min_zoom);
    }
}

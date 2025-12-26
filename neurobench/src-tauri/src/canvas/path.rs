//! Path Calculator - Edge path generation with bezier curves
//!
//! Calculates smooth bezier paths between nodes for edge rendering.

use super::types::CanvasNode;

/// Path calculator for edge routing
pub struct PathCalculator {
    /// Curvature factor for bezier curves
    curvature: f64,
}

impl PathCalculator {
    pub fn new() -> Self {
        Self {
            curvature: 0.5,
        }
    }
    
    /// Set curvature factor (0.0 = straight, 1.0 = very curved)
    pub fn set_curvature(&mut self, curvature: f64) {
        self.curvature = curvature.clamp(0.0, 1.0);
    }
    
    /// Calculate bezier path between two nodes
    pub fn calculate(&self, source: &CanvasNode, target: &CanvasNode) -> String {
        let (sx, sy) = source.output_port();
        let (tx, ty) = target.input_port();
        
        self.bezier_path(sx, sy, tx, ty)
    }
    
    /// Generate cubic bezier SVG path
    pub fn bezier_path(&self, sx: f64, sy: f64, tx: f64, ty: f64) -> String {
        let dy = (ty - sy).abs();
        let dx = (tx - sx).abs();
        
        // Control point offset based on distance
        let offset = (dy * self.curvature).max(40.0).min(150.0);
        
        // Handle edge cases (horizontal or very short)
        if dy < 20.0 && dx > 100.0 {
            // Horizontal edge - use S-curve
            let mid_y = (sy + ty) / 2.0;
            return format!(
                "M{:.1},{:.1} C{:.1},{:.1} {:.1},{:.1} {:.1},{:.1}",
                sx, sy,
                sx, mid_y,
                tx, mid_y,
                tx, ty
            );
        }
        
        // Standard vertical-ish bezier
        let c1y = sy + offset;
        let c2y = ty - offset;
        
        format!(
            "M{:.1},{:.1} C{:.1},{:.1} {:.1},{:.1} {:.1},{:.1}",
            sx, sy,      // Start point
            sx, c1y,     // Control point 1
            tx, c2y,     // Control point 2
            tx, ty       // End point
        )
    }
    
    /// Generate straight line path
    pub fn straight_path(&self, sx: f64, sy: f64, tx: f64, ty: f64) -> String {
        format!("M{:.1},{:.1} L{:.1},{:.1}", sx, sy, tx, ty)
    }
    
    /// Generate orthogonal (right-angle) path
    pub fn orthogonal_path(&self, sx: f64, sy: f64, tx: f64, ty: f64) -> String {
        let mid_y = (sy + ty) / 2.0;
        
        format!(
            "M{:.1},{:.1} L{:.1},{:.1} L{:.1},{:.1} L{:.1},{:.1}",
            sx, sy,       // Start
            sx, mid_y,    // Down from source
            tx, mid_y,    // Across
            tx, ty        // Up to target
        )
    }
    
    /// Calculate distance from point to edge
    pub fn distance_to_edge(&self, px: f64, py: f64, sx: f64, sy: f64, tx: f64, ty: f64) -> f64 {
        // For bezier, approximate by checking distance to line and midpoint
        let mid_x = (sx + tx) / 2.0;
        let mid_y = (sy + ty) / 2.0;
        
        // Distance to midpoint
        let dist_mid = ((px - mid_x).powi(2) + (py - mid_y).powi(2)).sqrt();
        
        // Distance to line segment
        let dist_line = self.point_to_line_distance(px, py, sx, sy, tx, ty);
        
        dist_mid.min(dist_line)
    }
    
    /// Distance from point to line segment
    fn point_to_line_distance(&self, px: f64, py: f64, x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
        let dx = x2 - x1;
        let dy = y2 - y1;
        let len_sq = dx * dx + dy * dy;
        
        if len_sq == 0.0 {
            return ((px - x1).powi(2) + (py - y1).powi(2)).sqrt();
        }
        
        let t = ((px - x1) * dx + (py - y1) * dy) / len_sq;
        let t = t.clamp(0.0, 1.0);
        
        let proj_x = x1 + t * dx;
        let proj_y = y1 + t * dy;
        
        ((px - proj_x).powi(2) + (py - proj_y).powi(2)).sqrt()
    }
    
    /// Calculate arrowhead points
    pub fn arrowhead(&self, tx: f64, ty: f64, angle: f64, size: f64) -> [(f64, f64); 3] {
        let angle_rad = angle.to_radians();
        let half_angle = 25.0_f64.to_radians();
        
        let left_angle = angle_rad + std::f64::consts::PI - half_angle;
        let right_angle = angle_rad + std::f64::consts::PI + half_angle;
        
        [
            (tx, ty), // Tip
            (tx + size * left_angle.cos(), ty + size * left_angle.sin()),
            (tx + size * right_angle.cos(), ty + size * right_angle.sin()),
        ]
    }
    
    /// Get the angle of approach to the target
    pub fn approach_angle(&self, sx: f64, sy: f64, tx: f64, ty: f64) -> f64 {
        (ty - sy).atan2(tx - sx).to_degrees()
    }
}

impl Default for PathCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::types::NodeType;
    
    #[test]
    fn test_bezier_path() {
        let calc = PathCalculator::new();
        
        let source = CanvasNode::new("s".into(), "Source".into(), NodeType::Process, 0.0, 0.0);
        let target = CanvasNode::new("t".into(), "Target".into(), NodeType::Process, 0.0, 200.0);
        
        let path = calc.calculate(&source, &target);
        
        assert!(path.starts_with("M"));
        assert!(path.contains("C"));
    }
    
    #[test]
    fn test_distance_to_edge() {
        let calc = PathCalculator::new();
        
        // Point on the line
        let dist = calc.distance_to_edge(50.0, 50.0, 0.0, 0.0, 100.0, 100.0);
        assert!(dist < 1.0);
        
        // Point far from line
        let dist = calc.distance_to_edge(0.0, 100.0, 0.0, 0.0, 100.0, 0.0);
        assert!(dist > 90.0);
    }
}

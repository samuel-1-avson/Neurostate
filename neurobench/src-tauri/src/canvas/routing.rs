//! Edge Routing - Advanced edge routing algorithms
//!
//! Provides orthogonal routing with obstacle avoidance, waypoints, and edge bundling.

use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet, BinaryHeap};
use std::cmp::Ordering;

/// Edge routing style
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum RoutingStyle {
    /// Smooth bezier curves
    #[default]
    Bezier,
    /// Right-angle paths
    Orthogonal,
    /// Straight lines
    Straight,
    /// Orthogonal with rounded corners
    OrthogonalRounded,
}

/// A waypoint on an edge path
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Waypoint {
    pub x: f64,
    pub y: f64,
    /// Whether this is a user-defined waypoint (vs auto-generated)
    pub manual: bool,
}

impl Waypoint {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y, manual: false }
    }

    pub fn manual(x: f64, y: f64) -> Self {
        Self { x, y, manual: true }
    }
}

/// Rectangle obstacle for routing
#[derive(Debug, Clone)]
pub struct Obstacle {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub id: String,
}

impl Obstacle {
    pub fn new(id: impl Into<String>, x: f64, y: f64, w: f64, h: f64) -> Self {
        Self {
            id: id.into(),
            x,
            y,
            width: w,
            height: h,
        }
    }

    /// Check if point is inside obstacle (with margin)
    pub fn contains(&self, px: f64, py: f64, margin: f64) -> bool {
        px >= self.x - margin && px <= self.x + self.width + margin &&
        py >= self.y - margin && py <= self.y + self.height + margin
    }

    /// Check if line segment intersects obstacle
    pub fn intersects_line(&self, x1: f64, y1: f64, x2: f64, y2: f64, margin: f64) -> bool {
        let rect_left = self.x - margin;
        let rect_right = self.x + self.width + margin;
        let rect_top = self.y - margin;
        let rect_bottom = self.y + self.height + margin;

        // Check if either endpoint is inside
        if self.contains(x1, y1, margin) || self.contains(x2, y2, margin) {
            return true;
        }

        // Check line-rectangle intersection using Liang-Barsky
        let dx = x2 - x1;
        let dy = y2 - y1;

        let p = [-dx, dx, -dy, dy];
        let q = [
            x1 - rect_left,
            rect_right - x1,
            y1 - rect_top,
            rect_bottom - y1,
        ];

        let mut t_min = 0.0_f64;
        let mut t_max = 1.0_f64;

        for i in 0..4 {
            if p[i].abs() < 1e-10 {
                if q[i] < 0.0 {
                    return false;
                }
            } else {
                let t = q[i] / p[i];
                if p[i] < 0.0 {
                    t_min = t_min.max(t);
                } else {
                    t_max = t_max.min(t);
                }
            }
        }

        t_min <= t_max
    }

    /// Get corner points for routing around
    pub fn corners(&self, margin: f64) -> [(f64, f64); 4] {
        [
            (self.x - margin, self.y - margin),
            (self.x + self.width + margin, self.y - margin),
            (self.x + self.width + margin, self.y + self.height + margin),
            (self.x - margin, self.y + self.height + margin),
        ]
    }
}

/// A* pathfinding node
#[derive(Clone)]
struct PathNode {
    x: f64,
    y: f64,
    g_cost: f64,
    h_cost: f64,
    parent: Option<Box<PathNode>>,
}

impl PathNode {
    fn f_cost(&self) -> f64 {
        self.g_cost + self.h_cost
    }
}

impl PartialEq for PathNode {
    fn eq(&self, other: &Self) -> bool {
        (self.x - other.x).abs() < 1.0 && (self.y - other.y).abs() < 1.0
    }
}

impl Eq for PathNode {}

impl Ord for PathNode {
    fn cmp(&self, other: &Self) -> Ordering {
        other.f_cost().partial_cmp(&self.f_cost()).unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for PathNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Edge router with multiple algorithms
pub struct EdgeRouter {
    /// Grid size for pathfinding
    grid_size: f64,
    /// Margin around obstacles
    margin: f64,
    /// Corner radius for rounded orthogonal
    corner_radius: f64,
}

impl EdgeRouter {
    pub fn new() -> Self {
        Self {
            grid_size: 20.0,
            margin: 15.0,
            corner_radius: 10.0,
        }
    }

    /// Set grid size for pathfinding
    pub fn with_grid_size(mut self, size: f64) -> Self {
        self.grid_size = size;
        self
    }

    /// Set margin around obstacles
    pub fn with_margin(mut self, margin: f64) -> Self {
        self.margin = margin;
        self
    }

    /// Generate orthogonal path avoiding obstacles
    pub fn orthogonal_route(
        &self,
        sx: f64,
        sy: f64,
        tx: f64,
        ty: f64,
        obstacles: &[Obstacle],
    ) -> Vec<(f64, f64)> {
        // Try simple L-path first
        if let Some(path) = self.try_l_path(sx, sy, tx, ty, obstacles) {
            return path;
        }

        // Try Z-path (with middle horizontal)
        if let Some(path) = self.try_z_path(sx, sy, tx, ty, obstacles) {
            return path;
        }

        // Fall back to A* pathfinding
        self.a_star_route(sx, sy, tx, ty, obstacles)
    }

    /// Try simple L-shaped path
    fn try_l_path(
        &self,
        sx: f64,
        sy: f64,
        tx: f64,
        ty: f64,
        obstacles: &[Obstacle],
    ) -> Option<Vec<(f64, f64)>> {
        // L-path going down then across
        let mid = (sx, ty);
        if !self.path_blocked(sx, sy, mid.0, mid.1, obstacles) &&
           !self.path_blocked(mid.0, mid.1, tx, ty, obstacles) {
            return Some(vec![(sx, sy), mid, (tx, ty)]);
        }

        // L-path going across then down
        let mid = (tx, sy);
        if !self.path_blocked(sx, sy, mid.0, mid.1, obstacles) &&
           !self.path_blocked(mid.0, mid.1, tx, ty, obstacles) {
            return Some(vec![(sx, sy), mid, (tx, ty)]);
        }

        None
    }

    /// Try Z-shaped path
    fn try_z_path(
        &self,
        sx: f64,
        sy: f64,
        tx: f64,
        ty: f64,
        obstacles: &[Obstacle],
    ) -> Option<Vec<(f64, f64)>> {
        let mid_y = (sy + ty) / 2.0;
        let p1 = (sx, mid_y);
        let p2 = (tx, mid_y);

        if !self.path_blocked(sx, sy, p1.0, p1.1, obstacles) &&
           !self.path_blocked(p1.0, p1.1, p2.0, p2.1, obstacles) &&
           !self.path_blocked(p2.0, p2.1, tx, ty, obstacles) {
            return Some(vec![(sx, sy), p1, p2, (tx, ty)]);
        }

        let mid_x = (sx + tx) / 2.0;
        let p1 = (mid_x, sy);
        let p2 = (mid_x, ty);

        if !self.path_blocked(sx, sy, p1.0, p1.1, obstacles) &&
           !self.path_blocked(p1.0, p1.1, p2.0, p2.1, obstacles) &&
           !self.path_blocked(p2.0, p2.1, tx, ty, obstacles) {
            return Some(vec![(sx, sy), p1, p2, (tx, ty)]);
        }

        None
    }

    /// Check if path segment is blocked
    fn path_blocked(&self, x1: f64, y1: f64, x2: f64, y2: f64, obstacles: &[Obstacle]) -> bool {
        obstacles.iter().any(|obs| obs.intersects_line(x1, y1, x2, y2, self.margin))
    }

    /// A* pathfinding for complex obstacle avoidance
    fn a_star_route(
        &self,
        sx: f64,
        sy: f64,
        tx: f64,
        ty: f64,
        obstacles: &[Obstacle],
    ) -> Vec<(f64, f64)> {
        let mut open_set = BinaryHeap::new();
        let mut closed_set = HashSet::new();

        let start = PathNode {
            x: sx,
            y: sy,
            g_cost: 0.0,
            h_cost: self.heuristic(sx, sy, tx, ty),
            parent: None,
        };

        open_set.push(start);

        let directions = [
            (0.0, -self.grid_size),   // Up
            (0.0, self.grid_size),    // Down
            (-self.grid_size, 0.0),   // Left
            (self.grid_size, 0.0),    // Right
        ];

        let mut iterations = 0;
        let max_iterations = 1000;

        while let Some(current) = open_set.pop() {
            iterations += 1;
            if iterations > max_iterations {
                break;
            }

            // Goal reached
            if (current.x - tx).abs() < self.grid_size && (current.y - ty).abs() < self.grid_size {
                return self.reconstruct_path(current, tx, ty);
            }

            let key = (
                (current.x / self.grid_size) as i64,
                (current.y / self.grid_size) as i64,
            );
            if closed_set.contains(&key) {
                continue;
            }
            closed_set.insert(key);

            for (dx, dy) in &directions {
                let nx = current.x + dx;
                let ny = current.y + dy;

                // Check if blocked
                if self.point_blocked(nx, ny, obstacles) {
                    continue;
                }

                let new_key = ((nx / self.grid_size) as i64, (ny / self.grid_size) as i64);
                if closed_set.contains(&new_key) {
                    continue;
                }

                let g_cost = current.g_cost + self.grid_size;
                let h_cost = self.heuristic(nx, ny, tx, ty);

                let neighbor = PathNode {
                    x: nx,
                    y: ny,
                    g_cost,
                    h_cost,
                    parent: Some(Box::new(current.clone())),
                };

                open_set.push(neighbor);
            }
        }

        // Fallback: direct path
        vec![(sx, sy), (tx, ty)]
    }

    fn heuristic(&self, x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
        // Manhattan distance for orthogonal movement
        (x2 - x1).abs() + (y2 - y1).abs()
    }

    fn point_blocked(&self, x: f64, y: f64, obstacles: &[Obstacle]) -> bool {
        obstacles.iter().any(|obs| obs.contains(x, y, self.margin))
    }

    fn reconstruct_path(&self, end: PathNode, tx: f64, ty: f64) -> Vec<(f64, f64)> {
        let mut path = Vec::new();
        let mut current = Some(Box::new(end));

        while let Some(node) = current {
            path.push((node.x, node.y));
            current = node.parent;
        }

        path.reverse();
        
        // Add target point
        if let Some(last) = path.last() {
            if (last.0 - tx).abs() > 1.0 || (last.1 - ty).abs() > 1.0 {
                path.push((tx, ty));
            }
        }

        // Simplify path (remove collinear points)
        self.simplify_path(path)
    }

    fn simplify_path(&self, path: Vec<(f64, f64)>) -> Vec<(f64, f64)> {
        if path.len() < 3 {
            return path;
        }

        let mut simplified = vec![path[0]];

        for i in 1..path.len() - 1 {
            let prev = simplified.last().unwrap();
            let curr = path[i];
            let next = path[i + 1];

            // Check if collinear (same direction)
            let dx1 = curr.0 - prev.0;
            let dy1 = curr.1 - prev.1;
            let dx2 = next.0 - curr.0;
            let dy2 = next.1 - curr.1;

            // Not collinear if direction changes
            if (dx1.abs() > 0.1 && dx2.abs() < 0.1) || 
               (dy1.abs() > 0.1 && dy2.abs() < 0.1) ||
               (dx1.abs() < 0.1 && dx2.abs() > 0.1) ||
               (dy1.abs() < 0.1 && dy2.abs() > 0.1) {
                simplified.push(curr);
            }
        }

        simplified.push(*path.last().unwrap());
        simplified
    }

    /// Convert path points to SVG path string
    pub fn path_to_svg(&self, points: &[(f64, f64)], style: RoutingStyle) -> String {
        if points.is_empty() {
            return String::new();
        }

        if points.len() == 1 {
            return format!("M{:.1},{:.1}", points[0].0, points[0].1);
        }

        match style {
            RoutingStyle::Straight => self.straight_svg(points),
            RoutingStyle::Orthogonal => self.orthogonal_svg(points),
            RoutingStyle::OrthogonalRounded => self.orthogonal_rounded_svg(points),
            RoutingStyle::Bezier => self.bezier_svg(points),
        }
    }

    fn straight_svg(&self, points: &[(f64, f64)]) -> String {
        let mut path = format!("M{:.1},{:.1}", points[0].0, points[0].1);
        for p in &points[1..] {
            path.push_str(&format!(" L{:.1},{:.1}", p.0, p.1));
        }
        path
    }

    fn orthogonal_svg(&self, points: &[(f64, f64)]) -> String {
        let mut path = format!("M{:.1},{:.1}", points[0].0, points[0].1);
        for p in &points[1..] {
            path.push_str(&format!(" L{:.1},{:.1}", p.0, p.1));
        }
        path
    }

    fn orthogonal_rounded_svg(&self, points: &[(f64, f64)]) -> String {
        if points.len() < 3 {
            return self.orthogonal_svg(points);
        }

        let mut path = format!("M{:.1},{:.1}", points[0].0, points[0].1);
        let r = self.corner_radius;

        for i in 1..points.len() - 1 {
            let prev = points[i - 1];
            let curr = points[i];
            let next = points[i + 1];

            // Direction vectors
            let d1 = (curr.0 - prev.0, curr.1 - prev.1);
            let d2 = (next.0 - curr.0, next.1 - curr.1);

            // Normalize and clamp radius
            let len1 = (d1.0.powi(2) + d1.1.powi(2)).sqrt();
            let len2 = (d2.0.powi(2) + d2.1.powi(2)).sqrt();
            let actual_r = r.min(len1 / 2.0).min(len2 / 2.0);

            if actual_r < 1.0 {
                path.push_str(&format!(" L{:.1},{:.1}", curr.0, curr.1));
                continue;
            }

            // Points before and after corner
            let before = (
                curr.0 - actual_r * d1.0 / len1,
                curr.1 - actual_r * d1.1 / len1,
            );
            let after = (
                curr.0 + actual_r * d2.0 / len2,
                curr.1 + actual_r * d2.1 / len2,
            );

            path.push_str(&format!(" L{:.1},{:.1}", before.0, before.1));
            path.push_str(&format!(" Q{:.1},{:.1} {:.1},{:.1}", curr.0, curr.1, after.0, after.1));
        }

        let last = points.last().unwrap();
        path.push_str(&format!(" L{:.1},{:.1}", last.0, last.1));
        path
    }

    fn bezier_svg(&self, points: &[(f64, f64)]) -> String {
        if points.len() < 2 {
            return self.straight_svg(points);
        }

        let start = points[0];
        let end = points[points.len() - 1];
        
        let dy = (end.1 - start.1).abs();
        let offset = (dy * 0.5).max(40.0).min(150.0);

        format!(
            "M{:.1},{:.1} C{:.1},{:.1} {:.1},{:.1} {:.1},{:.1}",
            start.0, start.1,
            start.0, start.1 + offset,
            end.0, end.1 - offset,
            end.0, end.1
        )
    }
}

impl Default for EdgeRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_route() {
        let router = EdgeRouter::new();
        let path = router.orthogonal_route(0.0, 0.0, 100.0, 100.0, &[]);
        assert!(!path.is_empty());
    }

    #[test]
    fn test_obstacle_avoidance() {
        let router = EdgeRouter::new();
        let obstacles = vec![
            Obstacle::new("o1", 40.0, 40.0, 30.0, 30.0),
        ];
        let path = router.orthogonal_route(0.0, 50.0, 100.0, 50.0, &obstacles);
        assert!(path.len() > 2); // Should route around
    }

    #[test]
    fn test_svg_generation() {
        let router = EdgeRouter::new();
        let points = vec![(0.0, 0.0), (50.0, 0.0), (50.0, 100.0), (100.0, 100.0)];
        
        let svg = router.path_to_svg(&points, RoutingStyle::Orthogonal);
        assert!(svg.starts_with("M"));
        
        let svg_rounded = router.path_to_svg(&points, RoutingStyle::OrthogonalRounded);
        assert!(svg_rounded.contains("Q")); // Has quadratic curves
    }
}

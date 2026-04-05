use super::window::Rect;
use std::collections::HashMap;

/// Manages interactive regions for app widgets
pub struct RegionManager {
    /// Map of app_id.widget_id -> Rect
    regions: HashMap<String, Rect>,
}

impl RegionManager {
    pub fn new() -> Self {
        Self {
            regions: HashMap::new(),
        }
    }

    /// Register a widget as interactive
    pub fn add_region(&mut self, widget_key: String, rect: Rect) {
        self.regions.insert(widget_key, rect);
    }

    /// Remove a widget's interactive region
    pub fn remove_region(&mut self, widget_key: &str) {
        self.regions.remove(widget_key);
    }

    /// Get all current interactive regions as a Vec
    pub fn get_all_regions(&self) -> Vec<Rect> {
        self.regions.values().cloned().collect()
    }

    /// Update a widget's region (e.g., after drag/resize)
    pub fn update_region(&mut self, widget_key: String, rect: Rect) {
        self.regions.insert(widget_key, rect);
    }

    /// Clear all regions
    pub fn clear(&mut self) {
        self.regions.clear();
    }
}

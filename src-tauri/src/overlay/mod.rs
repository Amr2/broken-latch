// Overlay module - handles transparent overlay windows
// Implemented in Task 02

pub mod window;
pub mod region;

pub use window::{OverlayWindow, Rect};
pub use region::RegionManager;

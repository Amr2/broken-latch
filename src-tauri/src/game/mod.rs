// Game lifecycle detection module

pub mod detect;
pub mod focus;
pub mod lcu;
pub mod session;

pub use detect::start_detector;
pub use focus::start_focus_monitor;

// DirectX hook module
// Implemented in Task 03

pub mod injector;
pub mod pipe;

pub use injector::inject_into_league;
pub use pipe::{PipeMessage, PipeServer};

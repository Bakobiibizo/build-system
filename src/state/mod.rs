pub mod error;
pub mod types;
pub mod manager;
pub mod dependency;

pub use manager::StateManager;

#[cfg(test)]
mod tests;

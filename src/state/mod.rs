pub mod conflict;
pub mod lock;
pub mod versioned;

// Re-export versioned state as the main HeimdalState type
pub use versioned::HeimdalState;

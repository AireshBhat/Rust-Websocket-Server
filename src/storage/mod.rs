// Export storage traits
pub mod traits;
// pub mod postgres;
// pub mod redis;
pub mod memory;

// Re-export traits for easier importing
pub use traits::user::UserStorage;
pub use traits::network::NetworkStorage; 
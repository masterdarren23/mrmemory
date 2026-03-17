mod auth;
mod health;
mod memories;

pub use auth::create_key;
pub use health::health_routes;
pub use memories::memory_routes;

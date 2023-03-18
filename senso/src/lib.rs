pub mod connector;
pub mod db;
pub mod response;
pub mod urls;

#[allow(dead_code)]
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

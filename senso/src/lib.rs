pub mod connector;
pub mod db;
pub mod request;
pub mod response;
pub mod urls;
pub mod yp;

#[allow(dead_code)]
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

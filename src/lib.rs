use once_cell::sync::Lazy;

pub mod config;
pub mod models;
// pub mod api;
pub mod repo;
pub mod db;
pub mod error;

pub static CONFIG: Lazy<config::Config> = Lazy::new(|| {
    config::Config::from_env().unwrap()
});

pub static REQWEST_CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
    reqwest::Client::new()
});

// Set your Redis URL (from env or hardcoded)
// pub static REDIS_CLIENT: Lazy<redis::Client> = Lazy::new(|| {
//     redis::Client::open(CONFIG.db_url.clone()).expect("Invalid Redis URL")
// });

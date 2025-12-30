mod auth;
mod config;
mod db;
mod models;
mod routes;
mod utils;

use config::Config;
use db::Db;
use rocket::{launch, routes, fairing::AdHoc};
use rocket_db_pools::Database;

#[launch]
async fn rocket() -> _ {
    // Load configuration
    let config = Config::from_env()
        .or_else(|_| Config::from_file("config.toml"))
        .expect("Failed to load configuration");

    // Build Rocket instance
    let figment = rocket::Config::figment()
        .merge(("address", config.server.host.clone()))
        .merge(("port", config.server.port))
        .merge(("databases.postgres.url", config.database.url.clone()));

    rocket::custom(figment)
        .attach(Db::init())
        .attach(AdHoc::try_on_ignite("Run Migrations", |rocket| async {
            use std::ops::Deref;
            
            match Db::fetch(&rocket) {
                Some(db) => {
                    match sqlx::query(db::migrations::INIT_SQL)
                        .execute(db.deref())
                        .await
                    {
                        Ok(_) => Ok(rocket),
                        Err(e) => {
                            eprintln!("Failed to run migrations: {}", e);
                            Err(rocket)
                        }
                    }
                }
                None => {
                    eprintln!("Failed to get database connection");
                    Err(rocket)
                }
            }
        }))
        .manage(config.clone())
        .mount("/auth", routes![
            routes::register,
            routes::login,
            routes::refresh_token,
        ])
        .mount("/auth/oauth", routes![
            routes::google_authorize,
            routes::google_callback,
            routes::microsoft_authorize,
            routes::microsoft_callback,
            routes::github_authorize,
            routes::github_callback,
        ])
}

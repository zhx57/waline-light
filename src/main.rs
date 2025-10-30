use tracing::level_filters::LevelFilter;
use tracing_subscriber::{EnvFilter, filter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

mod app;
mod components;
mod config;
mod entities;
mod error;
mod helpers;
mod locales;
mod middlewares;
mod migration;
mod prelude;
mod repository;
mod response;
mod traits;
mod types;

#[actix_web::main]
async fn main() -> Result<(), error::AppError> {
  let target_filter = filter::Targets::new()
    .with_default(LevelFilter::TRACE)
    .with_target("sqlx::query", LevelFilter::OFF)
    .with_target("html5ever", LevelFilter::OFF)
    .with_target("rustls", LevelFilter::OFF);
  let env_filter = EnvFilter::try_from_default_env()
    .or_else(|_| EnvFilter::try_new("info"))
    .unwrap();
  tracing_subscriber::registry()
    .with(tracing_subscriber::fmt::layer().with_timer(fmt::time::LocalTime::rfc_3339()))
    .with(target_filter)
    .with(env_filter)
    .init();
  app::start().await
}

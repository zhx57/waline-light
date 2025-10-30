pub mod handler;
mod model;
pub mod service;

use actix_web::web::ServiceConfig;

pub fn config(cfg: &mut ServiceConfig) {
  cfg.service(handler::ui_profile_page);
  cfg.service(handler::ui_login_page);
  cfg.service(handler::ui_migration_page);
  cfg.service(handler::ui_user_page);
  cfg.service(handler::ui_forgot_page);
}

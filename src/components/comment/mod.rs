mod handler;
pub mod model;
mod service;

use actix_web::web::ServiceConfig;

pub fn config(cfg: &mut ServiceConfig) {
  cfg.service(handler::get_comment_info);
  cfg.service(handler::create_comment);
  cfg.service(handler::delete_comment);
  cfg.service(handler::update_comment);
}

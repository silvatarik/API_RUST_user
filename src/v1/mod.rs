

mod users;

use actix_web::web;
use actix_web::web::ServiceConfig;

pub fn service(cfg: &mut ServiceConfig){
    cfg.service(
        web::scope("v1").configure(users::service)
    );
}
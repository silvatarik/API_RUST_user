use actix_web::{HttpRequest, HttpResponse, web};
use actix_web::error::PathError;
use actix_web::web::{Path, PathConfig, ServiceConfig};
use uuid::Uuid;
use crate::RepositoryInjector;

const PATH : &str = "/user";
pub fn service(cfg : &mut  ServiceConfig){
    cfg.service(
        web::scope(PATH)
            .app_data(PathConfig::default().error_handler(path_config_handler))

            //Aqui irian todos las acciones o los verbos GET,POST ETC
            .route("/{user_id}", web::get().to(get_user))

            //Con esto ultimo el error handler no se aplicaria a todo
            // .service(web::resource("/{user_id}")
            //     .app_data(PathConfig::default().error_handler(|err, req| {
            //         actix_web::error::ErrorBadRequest(err)
            //     })).route(web::get().to(get_user)))
    );
}

fn path_config_handler(err: PathError, req: &HttpRequest) -> actix_web::Error{
    actix_web::error::ErrorBadRequest(err)
}

//Recibe un request  que posteriormente devolvera un user
async fn get_user(
    user_id: web::Path<Uuid>,
    repo: web::Data<RepositoryInjector>,
) -> HttpResponse {
    // if let Ok(parsed_user_id) = Uuid::parse_str(&user_id) {
    //     //El 0 se pone porque el repository injector es tipo tupla por tanto como solo tenemos un elemento
    //     //seria el 0  Ahora funcionaria sin el debido a que implementamos el trait Deref
    //     match repo.get_users(*&parsed_user_id) {
    //         Ok(user) => HttpResponse::Ok().json(user),
    //         Err(_) => HttpResponse::NotFound().body("Not found"),
    //     }
    // } else {
    //     HttpResponse::BadRequest().body("Invalid UUID")
    // }

        match repo.get_users(**&user_id) {
            Ok(user) => HttpResponse::Ok().json(user),
            Err(_) => HttpResponse::NotFound().body("Not found"),
        }
}
use std::sync::Arc;
use std::sync::atomic::{AtomicU16, Ordering};
use actix_web::{web, App, HttpRequest, HttpServer, Responder, HttpResponse};

mod user;
mod repository;
use repository::{MemoryRepository, Repository};
use uuid::Uuid;

//Esta funcion recoge del request un parametro name de la req
async fn greet(req: HttpRequest) -> impl Responder {
    let name = req.match_info().get("name").unwrap_or("World");
    format!("Hello {}!", &name)
}

//Recibe un request  que posteriormente devolvera un user
async fn get_user(user_id: web::Path<String>) -> HttpResponse {
    if let Ok(parsed_user_id) = Uuid::parse_str(&user_id) {
        let repo = MemoryRepository::default();
        match repo.get_users(*&parsed_user_id) {
            Ok(user) => HttpResponse::Ok().json(user),
            Err(_) => HttpResponse::NotFound().body("Not found"),
        }
    } else {
        HttpResponse::BadRequest().body("Invalid UUID")
    }
}

//el actix web pertenece al uruntime de tokio
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Se inicializa las variables de entorno
    dotenv::dotenv().ok();

    //Contruyendo el address
    let port = std::env::var("PORT").unwrap_or("8080".to_string());
    let address = format!("127.0.0.1:{}",port);

    let thread_counter = Arc::new(AtomicU16::new(1));
    HttpServer::new(move || {
        println!("Starting thread {}",
        thread_counter.fetch_add(1,Ordering::SeqCst));
        let thread_index = thread_counter.load(Ordering::SeqCst);
        App::new()
            //El service se suele usar para middleware y llevar un mejor control
            .service(web::resource("/user/{user_id}").route(web::get().to(
                get_user
            )))
            //Endpoints o rutas que usaran
            .route("/", web::get().to(|| HttpResponse::Ok().body("Hola Rust")))
            .route("/health", web::get().to(move || {
                HttpResponse::Ok()
                    .header("thread-id", thread_index.to_string())
                    .finish()
            }),
            )
            .route("/{name}", web::get().to(greet))
    })
        .bind(&address)
        .unwrap_or_else(|err| panic!("Port : {} - Error : {}", port, err))
        .run()
        .await
}

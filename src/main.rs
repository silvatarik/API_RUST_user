use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::Arc;

mod repository;
mod user;
mod health;
mod v1;
use crate::repository::RepositoryInjector;
use repository::{MemoryRepository};

//Esta funcion recoge del request un parametro name de la req
async fn greet(req: HttpRequest) -> impl Responder {
    let name = req.match_info().get("name").unwrap_or("World");
    format!("Hello {}!", &name)
}

//el actix web pertenece al uruntime de tokio
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Se inicializa las variables de entorno
    dotenv::dotenv().ok();

    //Contruyendo el address
    let port = std::env::var("PORT").unwrap_or("8080".to_string());
    let address = format!("127.0.0.1:{}", port);

    let thread_counter = Arc::new(AtomicU16::new(1));
    //Aqui se aplica el concepto de shadowing
    //let repo = web::Data::new(RepositoryInjector::new(MemoryRepository::default()));
    // let repo = RepositoryInjector::new(MemoryRepository::default());
    let repo = web::Data::new(MemoryRepository::default());

    HttpServer::new(move || {
        let thread_index = thread_counter.fetch_add(1, Ordering::SeqCst);
        println!("Starting thread {}", thread_index);

        App::new()
            //data permite compartir un estado
            .data(thread_index)
            .app_data(repo.clone())
            //El service se suele usar para middleware y llevar un mejor control
            .configure(v1::service::<MemoryRepository>)
            //Endpoints o rutas que usaran
            .route("/", web::get().to(|| HttpResponse::Ok().body("Hola Rust")))
            //el web::Data sirve a modo de extractor tal como el web::Path
            .configure(health::service)
            .route("/{name}", web::get().to(greet))
    })
    .bind(&address)
    .unwrap_or_else(|err| panic!("Port : {} - Error : {}", port, err))
    .run()
    .await
}

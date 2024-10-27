use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use dotenv::dotenv;
use mongodb::{options::ClientOptions, Client};
use std::env;
use std::sync::Arc;
use mongodb::bson::doc;
use env_logger;
use log::info;

async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("Server is running!")
}

async fn check_mongo(db_client: web::Data<Arc<Client>>) -> impl Responder {
    match db_client.database("admin").run_command(doc! {"ping": 1}, None).await {
        Ok(_) => HttpResponse::Ok().body("MongoDB connection is successful!"),
        Err(e) => HttpResponse::InternalServerError().body(format!("MongoDB connection failed: {:?}", e)),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok(); // Load env variables
    env_logger::init(); // Initialize logger
    info!("Starting server...");

    let server_address = env::var("SERVER_ADDRESS").unwrap_or_else(|_| "127.0.0.1:8080".to_string());
    let mongo_uri = env::var("MONGODB_URI").expect("MongoDB URI not set");

    // Set up MongoDB client
    let client_options = ClientOptions::parse(&mongo_uri).await.unwrap();
    let mongo_client = Client::with_options(client_options).unwrap();
    let mongo_client = Arc::new(mongo_client);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(mongo_client.clone()))
            .route("/", web::get().to(health_check))
            .route("/check_mongo", web::get().to(check_mongo))
    })
        .bind(&server_address)?
        .run()
        .await
}

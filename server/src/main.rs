mod handlers;
mod utils;
// mod unit_tests;

use std::env;
use std::sync::Arc;
use actix_web::{App, HttpServer, web};
use deadpool_postgres::{ManagerConfig, RecyclingMethod, Config, Runtime};
use tokio_postgres::NoTls;
use indexmap::IndexMap;
use tokio::sync::RwLock;
use handlers::routes as rt;
use utils::sql_executor::{SQLExecutor, ProdSQLExecutor};
use utils::app_state::{AppState, Cache};

#[tokio::main]
pub async fn main() -> std::io::Result<()> {

    println!("#=====================================#");
    println!("# Welcome to the Paidy Pantry Server! #");
    println!("#=====================================#\n");

    env_logger::init();

    let db_host = env::var("DB_HOST").expect("DB_HOST not set");
    let db_user = env::var("DB_USER").expect("DB_USER not set");
    let db_password = env::var("DB_PASSWORD").expect("DB_PASSWORD not set");
    let db_name = env::var("DB_NAME").expect("DB_NAME not set");
    let db_port = env::var("DB_PORT").expect("DB_PORT not set").parse::<u16>().expect("DB_PORT must be a valid number");

    let mut config = Config::new();
    config.host = Some(db_host);
    config.user = Some(db_user);
    config.password = Some(db_password);
    config.dbname = Some(db_name);
    config.port = Some(db_port);
    config.manager = Some(ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    });

    let db_pool = config.create_pool(Some(Runtime::Tokio1), NoTls).unwrap();
    let cache: Cache = Arc::new(RwLock::new(IndexMap::new()));
    let db_executor: Arc<dyn SQLExecutor + Send + Sync> = Arc::new(ProdSQLExecutor);
    let app_state: AppState = AppState { db_pool, db_executor, cache };

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .service(rt::create_items)
            .service(rt::read_table)
            .service(rt::delete_item)
    })
    .bind("0.0.0.0:8080")?
    .workers(10)
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::collections::HashMap;
    use tokio_postgres::NoTls;
    use tokio::sync::RwLock;
    use deadpool_postgres::Config;
    use deadpool_postgres::{Pool, Runtime};
    use async_trait::async_trait;
    use actix_web::{body::to_bytes, http::StatusCode, web, App, HttpResponse};
    use actix_web::test::{init_service, TestRequest, call_service};
    use actix_web::dev::ServiceResponse;
    use crate::utils::sql_executor::{SQLExecutor, SQLResult};
    use crate::utils::app_state::{AppState, Cache};
    use crate::handlers::response::process_and_send_response;
    use crate::handlers::routes::{create_items, read_table, delete_item};

    // Struct for mocking successful queries
    struct MockSQLExecutor;
    // Struct for mocking failed queries
    struct MockBadSQLExecutor;

    // SQLExecutor trait for mocking successful queries
    #[async_trait]
    impl SQLExecutor for MockSQLExecutor {
        async fn sql_exec(
            &self,
            _db_pool: Pool,
            statement: &str, 
            _params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
        ) -> Result<Option<HashMap<i32, (String, i32)>>, String> {
            if statement.starts_with("SELECT") {
                let mut table_data = HashMap::new();
                table_data.insert(1, ("item".to_string(), 5));
                Ok(Some(table_data))
            } else {
                Ok(None)
            }
        }
    }

    // SQLExecutor trait for mocking failed queries
    #[async_trait]
    impl SQLExecutor for MockBadSQLExecutor {
        async fn sql_exec(
            &self,
            _db_pool: Pool,
            _statement: &str, 
            _params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
        ) -> Result<Option<HashMap<i32, (String, i32)>>, String> {
            Err("Database Error!".to_string())
        }
    }
    
    // App state for successful queries
    async fn init_app_state() -> web::Data<AppState> {
        let mut config = Config::new();
            config.dbname = Some("db".to_string());
            config.user = Some("user".to_string());
            config.password = Some("password".to_string());
            config.host = Some("host".to_string());
            config.port = Some(5432);
        let pool = config.create_pool(Some(Runtime::Tokio1), NoTls).unwrap();
        let cache: Cache = Arc::new(RwLock::new(HashMap::new()));
        let db_executor: Arc<dyn SQLExecutor + Send + Sync> = Arc::new(MockSQLExecutor);
        web::Data::new(AppState {db_pool: pool, cache, db_executor})
    }

    // App state for failed queries
    async fn init_bad_app_state() -> web::Data<AppState> {
        let mut config = Config::new();
            config.dbname = Some("db".to_string());
            config.user = Some("user".to_string());
            config.password = Some("password".to_string());
            config.host = Some("host".to_string());
            config.port = Some(5432);
        let db_pool = config.create_pool(Some(Runtime::Tokio1), NoTls).unwrap();
        let db_executor: Arc<dyn SQLExecutor + Send + Sync> = Arc::new(MockBadSQLExecutor);
        let cache: Cache = Arc::new(RwLock::new(HashMap::new()));
        web::Data::new(AppState { db_pool, cache, db_executor })
    }

    #[actix_web::test]
    async fn test_create_items() {
        let app_state: web::Data<AppState> = init_app_state().await;
        let mut app = init_service(
            App::new()
                .app_data(app_state)
                .service(create_items)
        ).await;
        let request_data: web::Json<Vec<(String, i32)>> = web::Json(vec![("item1".to_string(), 5), ("item2".to_string(), 10)]);
        let req = TestRequest::post()
            .uri("/table/1")
            .set_json(&request_data)
            .to_request();
        let response: ServiceResponse = call_service(&mut app, req).await;
        assert!(response.status().is_success(), "Response was not successful");
        let body: actix_web::body::BoxBody = response.into_body();
        let bytes: web::Bytes = to_bytes(body).await.expect("Failed to read body");
        let response_message: String = String::from_utf8(bytes.to_vec()).expect("Response was not valid UTF-8");
        let expected_message: &str = "The following items have been added to table 1:\nitem1 - 5 minutes\nitem2 - 10 minutes\n";
        assert_eq!(response_message, expected_message);
    }

    #[actix_web::test]
    async fn test_read_table() {
        let app_state = init_app_state().await;
        let mut app = init_service(
            App::new()
                .app_data(app_state)
                .service(read_table)
        ).await;
        let request = TestRequest::get()
            .uri("/table/1")
            .to_request();
        let response = call_service(&mut app, request).await;
        assert!(response.status().is_success(), "Response was not successful");
        let body = response.into_body();
        let bytes: web::Bytes = to_bytes(body).await.expect("Failed to read body");
        let response_table_data: HashMap<i32, (String, i32)> = match serde_json::from_slice(&bytes) {
            Ok(hashmap) => hashmap,
            Err(_) => {panic!("Could not deserialize response!")},
        };
        let mut expected_table_data: HashMap<i32, (String, i32)> = HashMap::new();
        expected_table_data.insert(1, ("item".to_string(), 5));
        assert_eq!(response_table_data, expected_table_data);
    }

    #[actix_web::test]
    async fn test_delete_item() {
        let app_state = init_app_state().await;
        let mut app = init_service(
            App::new()
                .app_data(app_state)
                .service(delete_item)
        ).await;
        let req = TestRequest::delete()
            .uri("/table/1/item/2")
            .to_request();
        let response: ServiceResponse = call_service(&mut app, req).await;
        assert!(response.status().is_success(), "Response was not successful");
        let body: actix_web::body::BoxBody = response.into_body();
        let bytes: web::Bytes = to_bytes(body).await.expect("Failed to read body");
        let response_message = String::from_utf8(bytes.to_vec()).expect("Response was not valid UTF-8");
        let expected_message: &str = "Item deleted from table 1!";
        assert_eq!(response_message, expected_message);
    }

    #[actix_web::test]
    async fn test_create_items_fail() {
        let app_state = init_bad_app_state().await;
        let mut app = init_service(
            App::new()
                .app_data(app_state)
                .service(delete_item)
        ).await;
        let request_data = web::Json(vec![("mock_item_1".to_string(), 10)]);
        let req = TestRequest::delete()
            .uri("/table/1/item/2")
            .set_json(&request_data)
            .to_request();
        let response: ServiceResponse = call_service(&mut app, req).await;
        assert!(!response.status().is_success());
        let body: actix_web::body::BoxBody = response.into_body();
        let bytes = to_bytes(body).await.expect("Failed to read body");
        let response_message: String = String::from_utf8(bytes.to_vec()).expect("Response was not valid UTF-8");
        let expected_message: &str = "Database Error!";
        assert_eq!(response_message, expected_message);
    }

    #[actix_web::test]
    async fn test_read_table_fail() {
        let app_state: web::Data<AppState> = init_bad_app_state().await;
        let mut app = init_service(
            App::new()
                .app_data(app_state)
                .service(read_table)
        ).await;
        let req = TestRequest::get()
            .uri("/table/1")
            .to_request();
        let response = call_service(&mut app, req).await;
        assert!(!response.status().is_success());
        let body: actix_web::body::BoxBody = response.into_body();
        let bytes: web::Bytes = to_bytes(body).await.expect("Failed to read body");
        let response_message: String = String::from_utf8(bytes.to_vec()).expect("Response was not valid UTF-8");
        let expected_message: &str = "Database Error!";
        assert_eq!(response_message, expected_message);
    }

    #[actix_web::test]
    async fn test_delete_item_fail() {
        let app_state = init_bad_app_state().await;
        let mut app = init_service(
            App::new()
                .app_data(app_state)
                .service(delete_item)
        ).await;
        let req = TestRequest::delete()
            .uri("/table/1/item/2")
            .to_request();
        let response: ServiceResponse = call_service(&mut app, req).await;
        assert!(!response.status().is_success());
        let body: actix_web::body::BoxBody = response.into_body();
        let bytes: web::Bytes = to_bytes(body).await.expect("Failed to read body");
        let response_message: String = String::from_utf8(bytes.to_vec()).expect("Response was not valid UTF-8");
        let expected_message: &str = "Database Error!";
        assert_eq!(response_message, expected_message);
    }

    #[actix_web::test]
    async fn test_process_and_send_response_with_table_data() {
        let mut expected_table_data: HashMap<i32, (String, i32)> = HashMap::new();
        expected_table_data.insert(1, ("mock_item_1".to_string(), 5));
        expected_table_data.insert(2, ("mock_item_2".to_string(), 10));
        let sql_result: SQLResult = Ok(Some(expected_table_data.clone()));
        let response: HttpResponse = process_and_send_response(sql_result, None);
        assert_eq!(response.status(), StatusCode::OK);
        let body_bytes: web::Bytes = to_bytes(response.into_body()).await.unwrap();
        let response_table_data: HashMap<i32, (String, i32)> = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(response_table_data, expected_table_data);
    }

    #[actix_web::test]
    async fn test_process_and_send_response_with_message() {
        let expected_message: String = "Test message".to_string();
        let sql_result: SQLResult = Ok(None);
        let response: HttpResponse = process_and_send_response(sql_result, Some(expected_message.clone()));
        assert_eq!(response.status(), StatusCode::OK);
        let response_message: web::Bytes = to_bytes(response.into_body()).await.unwrap();
        assert_eq!(response_message, expected_message);
    }

    #[actix_web::test]
    async fn test_process_and_send_response_with_error() {
        let expected_message: String = "Test Error".to_string();
        let sql_result: SQLResult = Err(expected_message.clone());
        let response: HttpResponse = process_and_send_response(sql_result, None);
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let response_message: web::Bytes = to_bytes(response.into_body()).await.unwrap();
        assert_eq!(response_message, expected_message);
    }
}

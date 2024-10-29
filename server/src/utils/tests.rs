#[cfg(test)]
mod tests {
    use tokio_postgres::NoTls;
    use deadpool_postgres::Config;
    use std::sync::Arc;
    use std::collections::HashMap;
    use indexmap::IndexMap;
    use tokio::sync::RwLock;
    use deadpool_postgres::{Pool, Runtime};
    use async_trait::async_trait;
    use actix_web::web;
    use crate::utils::app_state::AppState;
    use crate::utils::sql_executor::SQLExecutor;

    struct MockSQLExecutor;

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

    async fn init_app_state() -> web::Data<AppState> {
        let mut config = Config::new();
            config.dbname = Some("db".to_string());
            config.user = Some("user".to_string());
            config.password = Some("password".to_string());
            config.host = Some("host".to_string());
            config.port = Some(5432);
        let db_pool = config.create_pool(Some(Runtime::Tokio1), NoTls).unwrap();
        let db_executor: Arc<dyn SQLExecutor + Send + Sync> = Arc::new(MockSQLExecutor);
        let cache = Arc::new(RwLock::new(IndexMap::new()));
        web::Data::new(AppState {
            db_pool, cache, db_executor,
        })
    }

    #[actix_web::test]
    async fn test_cache_create() {
        let app_state: web::Data<AppState> = init_app_state().await;
        let mut mock_table_data: HashMap<i32, (String, i32)> = HashMap::new();
        mock_table_data.insert(1, ("mock_item".to_string(), 10));
        app_state.cache_create(1, mock_table_data.clone()).await;
        let mut expected_cache: IndexMap<i32, HashMap<i32, (String, i32)>> = IndexMap::new();
        expected_cache.insert(1, mock_table_data);
        let result_cache: IndexMap<i32, HashMap<i32, (String, i32)>> = app_state.cache.read().await.clone();
        assert_eq!(result_cache, expected_cache)
    }

    #[actix_web::test]
    async fn test_cache_read_miss() {
        let app_state: web::Data<AppState> = init_app_state().await;
        let cache_read_result: Option<HashMap<i32, (String, i32)>> = app_state.cache_read(1).await;
        assert_eq!(cache_read_result, None)
    }

    #[actix_web::test]
    async fn test_cache_read_hit() {
        let app_state: web::Data<AppState> = init_app_state().await;
        let mut mock_table_data: HashMap<i32, (String, i32)> = HashMap::new();
        mock_table_data.insert(1, ("mock_item".to_string(), 10));
        let mock_table_data_clone: HashMap<i32, (String, i32)> = mock_table_data.clone();
        app_state.cache_create(1, mock_table_data).await;
        let cache_read_result: HashMap<i32, (String, i32)> = app_state.cache_read(1).await.unwrap();
        assert_eq!(cache_read_result, mock_table_data_clone)
    }

    #[actix_web::test]
    async fn test_cache_read_hit_reorder() {
        let app_state: web::Data<AppState> = init_app_state().await;
        let mut mock_table_data: HashMap<i32, (String, i32)> = HashMap::new();
        let mock_item_time: (String, i32) = ("mock_item".to_string(), 5);
        let mut cached_data: HashMap<i32, (String, i32)> = HashMap::new();
        let mut expected_cache: IndexMap<i32, HashMap<i32, (String, i32)>> = IndexMap::new();

        cached_data.insert(5, mock_item_time.clone());
        expected_cache.insert(5, cached_data.clone());
        cached_data.remove(&5);

        for item_id in 1..=5 {
            mock_table_data.insert(item_id, mock_item_time.clone());
            app_state.cache_create(item_id, mock_table_data.clone()).await;
            mock_table_data.remove(&item_id);
            if item_id != 1 && item_id != 5 {
                cached_data.insert(item_id, mock_item_time.clone());
                expected_cache.insert(item_id, cached_data.clone());
                cached_data.remove(&item_id);
            }
        }
        cached_data.insert(1, mock_item_time.clone());
        expected_cache.insert(1, cached_data);
        app_state.cache_read(1).await;
        let result_cache: IndexMap<i32, HashMap<i32, (String, i32)>> = app_state.cache.read().await.clone();
        assert_eq!(result_cache, expected_cache)
    }
    
    #[actix_web::test]
    async fn test_cache_delete() {
        let app_state: web::Data<AppState> = init_app_state().await;
        let mut mock_table_data: HashMap<i32, (String, i32)> = HashMap::new();
        mock_table_data.insert(1, ("mock_item".to_string(), 10));
        app_state.cache_create(1, mock_table_data.clone()).await;
        app_state.cache_create(2, mock_table_data.clone()).await;
        app_state.cache_delete(1).await;
        let mut expected_cache: IndexMap<i32, HashMap<i32, (String, i32)>> = IndexMap::new();
        expected_cache.insert(2, mock_table_data);
        let result_cache: IndexMap<i32, HashMap<i32, (String, i32)>> = app_state.cache.read().await.clone();
        assert_eq!(result_cache, expected_cache)
    }
}

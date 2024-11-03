use std::sync::Arc;
use log::info;
use tokio::sync::RwLock;
use std::collections::HashMap;
// use indexmap::IndexMap;
use deadpool_postgres::Pool;
use crate::utils::sql_executor::SQLExecutor;

pub type Cache = Arc<RwLock<HashMap<i32, HashMap<i32, (String, i32)>>>>;

/// A struct representing the app state
#[derive(Clone)]
pub struct AppState {
    /// Database connection pool to manage connections to the PostgreSQL database
    pub db_pool: Pool,
    /// Database execution struct for implementing the correct executor function
    pub db_executor: Arc<dyn SQLExecutor + Send + Sync>,
    // Application level cache
    pub cache: Cache,
}

impl AppState {
    /// Inserts table data into the cache
    /// 
    /// Arguments
    /// 
    /// * `table_no` - the table number corresponding to the table data
    /// * `table_data` - A hashmap between the item ids and tuples of (String, i32) representing item name and cooking time respectively
    pub async fn cache_create(&self, table_no: i32, table_data: HashMap<i32, (String, i32)>) {
        info!("Data for table {} added to cache", table_no);
        let mut lock_cache_write = self.cache.write().await;
        lock_cache_write.insert(table_no, table_data);
    }

    /// Returns the appropriate data upon a cache hit, otherwise None
    /// 
    /// # Arguments
    /// 
    /// * `table_no` - The associated table number to look for in the cache
    /// 
    /// # Returns
    /// 
    /// Returns:
    /// - Some(HashMap<i32, (String, i32)>) upon a cache hit, with keys representing item ids, 
    ///   and value (String, i32) represents item name and cooking time respectively.
    /// - Returns None upon a cache miss.
    pub async fn cache_read(&self, table_no: i32) -> Option<HashMap<i32, (String, i32)>>  {
        let lock_cache_read = self.cache.read().await;
        if let Some(table_data) = lock_cache_read.get(&table_no).cloned() {
            info!("CACHE HIT for table {}", table_no);
            return Some(table_data);
        }
        info!("CACHE MISS for table {}", table_no);
        None
    }

    /// Deletes data for an associated table number
    /// 
    /// # Arguments
    /// 
    /// * `table_no` - The table number for which the associated table data to be removed
    pub async fn cache_delete(&self, table_no: i32) {
        let lock_cache_read = self.cache.read().await;
        if lock_cache_read.contains_key(&table_no) {
            drop(lock_cache_read);
            let mut lock_cache_write = self.cache.write().await;
            lock_cache_write.remove(&table_no);
            info!("Removing table {} from cache", table_no);
        }
    }
}

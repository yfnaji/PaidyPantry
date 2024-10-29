use async_trait::async_trait;
use std::collections::HashMap;
use tokio_postgres::Row;
use deadpool_postgres::{Pool, Client};
use log::info;

pub type SQLResult = Result<Option<HashMap<i32, (String, i32)>>, String>;

/// A trait for executing SQL queries on PostgreSQL
#[async_trait]
pub trait SQLExecutor {
    /// Returns an outcome of the query run on the database
    /// 
    /// # Arguments
    /// 
    /// * `db_pool` - Connection pool for the database
    /// * `statement` - The SQL statement to be executed
    /// * `params` - The parameters required for the SQL query
    /// 
    /// # Returns
    /// 
    /// A `Result<Option<HashMap<i32, (String, i32)>>, String>`, where:
    /// * Returns Ok(Some(HashMap<i32, (String, i32)>)) for a successful SELECT query
    /// * Returns Ok(None) for a successful CREATE/DELETE query
    /// * Returns Err(String) upon error with the desciption of the issue
    async fn sql_exec(
        &self,
        db_pool: Pool,
        statement: &str, 
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> SQLResult;
}

// A struct for the production implementation of SQLExecutor
pub struct ProdSQLExecutor;
    /// Returns an outcome of the query run on the database.
    /// 
    /// Production implementation of SQLExecutor trait.
    /// 
    /// # Arguments
    /// 
    /// * `db_pool` - Connection pool for the database
    /// * `statement` - The SQL statement to be executed
    /// * `params` - The parameters required for the SQL query
    /// 
    /// # Returns
    /// 
    /// A `Result<Option<HashMap<i32, (String, i32)>>, String>`, where:
    /// * Returns Ok(Some(HashMap<i32, (String, i32)>)) for a successful SELECT query
    /// * Returns Ok(None) for a successful CREATE/DELETE query
    /// * Returns Err(String) upon error with the desciption of the issue
#[async_trait]
impl SQLExecutor for ProdSQLExecutor {
    async fn sql_exec(
        &self,
        db_pool: Pool,
        statement: &str, 
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> SQLResult {

        info!("Attempting database connection from pool...");
        let client: Client = match db_pool.get().await {
            Ok(client) => client,
            Err(e) => return Err(format!("Connection pool error: {}", e))
        };

        info!("Running SQL query...");
        if statement.starts_with("SELECT") {
            let rows: Vec<Row> = match client.query(statement, params).await {
                Ok(result) => result,
                Err(e) => return Err(format!("SQL Query Error: {}", e)),
            };
            let mut table_data: HashMap<i32, (String, i32)> = HashMap::new();
            for row in rows {
                table_data.insert(
                    row.get("item_id"),
                    (row.get("item"), row.get("time")),
                );
            }
            Ok(Some(table_data))
        } else {
            match client.execute(statement, params).await {
                Ok(_) => Ok(None),
                Err(e) => return Err(format!("SQL Execution Error: {}", e)),
            }
        }
    }
}

use log::{info, warn};
use actix_web::{get, post, delete, web, HttpResponse};
use crate::utils::sql_executor::SQLResult;
use crate::AppState;
use crate::handlers::response::process_and_send_response;

/// POST request handler for creating items for a table.
/// 
/// Inserts items and their cooking times into the database for a table number.
/// Will also remove data corresponsing to the table number from the cache.
/// 
/// # Arguments 
///
/// * `app_state` - Object containing shared resources in app.
/// * `path` - Path segment of URI parsed as i32 representing the table no.
/// * `request` - Request JSON body parsed as a vector of (String, i32) tuples, representing item name and time respectively.
/// 
/// # Returns
///
/// Returns:
/// * `HttpResponse` - An HTTP response with a String declaring the success or failure of the request.
#[post("/table/{table_no}")]
pub async fn create_items(
    app_state: web::Data<AppState>, path: web::Path<i32>, request: web::Json<Vec<(String, i32)>>
) -> HttpResponse {
    let table_no: i32 = path.into_inner();
    info!("Request received to create items for table {}", table_no);
    let items_times: Vec<(String, i32)> = request.into_inner();
    let mut success_message: String = format!("The following items have been added to table {}:\n", table_no);
    let mut values: Vec<String> = Vec::new();
    let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![&table_no as &(dyn tokio_postgres::types::ToSql + Sync)];
    
    for (i, (item, time)) in items_times.iter().enumerate() {
        values.push(format!("($1, ${}, ${})", 2 * i + 2, 2 * i + 3));
        params.push(item as &(dyn tokio_postgres::types::ToSql + Sync));
        params.push(time as &(dyn tokio_postgres::types::ToSql + Sync));
        success_message.push_str(&format!("{} - {} minutes\n", item.clone(), time.clone()));
    }
    
    info!("Attempting to create items...");
    let statement: &str  = &format!("INSERT INTO items (table_no, item, time) VALUES {}", values.join(", "));
    let sql_result: SQLResult = app_state.db_executor.sql_exec(
        app_state.db_pool.clone(), 
        statement, 
        &params
    ).await;

    if let Err(error) = &sql_result {
        warn!("Could not create items for table {}!", table_no);
        warn!("{}", error);
    } else {
        info!("{}", success_message);
    }
    
    app_state.cache_delete(table_no).await;
    process_and_send_response(sql_result, Some(success_message))
}

/// GET request handler for reading items on a table
/// 
/// Checks the cache first for the table number. If table number exists in the cache, then
/// return the respond with the associated data. If not, the table data will be queried from
/// the database and will subsequently be inserted into the cache. 
/// 
/// * `app_state` - Object containing shared resources in app
/// * `path` - Path segment of URI parsed as i32 representing the table number
/// 
/// Returns:
/// * `HttpResponse` - An HTTP response with a HashMap<i32, (String, i32)> where the keys are the item ids 
/// and the values are the item names and cooking times respectively or a message in the form of a String if
/// an error has occured.
#[get("/table/{table_no}")]
pub async fn read_table(app_state: web::Data<AppState>, path: web::Path<i32>) -> HttpResponse {
    let table_no: i32 = path.into_inner();
    info!("Request received to read table {}", table_no);
    if let Some(table_data) = app_state.cache_read(table_no).await {
        return process_and_send_response(Ok(Some(table_data)), None);
    }
    let statement: &str = "SELECT * FROM items WHERE table_no = $1 ORDER BY item_id ASC";
    let params: &[&(dyn tokio_postgres::types::ToSql + Sync)] = &[
        &table_no as &(dyn tokio_postgres::types::ToSql + Sync),
    ];
    info!("Reading items for table {}", table_no);
    let sql_result: SQLResult = app_state.db_executor.sql_exec(
        app_state.db_pool.clone(), 
        statement, 
        params
    ).await;
    
    if let Ok(Some(table_data)) = &sql_result {
        app_state.cache_create(table_no, table_data.clone()).await;
    } else {
        warn!("Could not read table {}!", table_no);
        if let Err(error) = &sql_result {
            warn!("{}", error);
        }
    }

    process_and_send_response(sql_result, None)
}

/// DELETE request handler to delete an item from a table.
/// 
/// Deletes an item entry for a table number. Will also remove the data associated to the table number from the cache.
/// 
/// * `app_state` - Object containing shared resources in app.
/// * `path` - Path segment of URI parsed as a tuple (i32, i32) representing the table number and item id respectively.
/// 
/// Returns:
/// * `HttpResponse` - An HTTP response with a String declaring the success or failure of the request.
#[delete("/table/{table_no}/item/{item_id}")]
pub async fn delete_item(app_state: web::Data<AppState>, path: web::Path<(i32, i32)>) -> HttpResponse {
    let (table_no, item_id) = path.into_inner(); 
    info!("Request received to delete item from table {}", table_no);
    let statement: &str = "DELETE FROM items WHERE item_id = $1 AND table_no = $2";
    let params: &[&(dyn tokio_postgres::types::ToSql + Sync)] = &[
        &item_id as &(dyn tokio_postgres::types::ToSql + Sync),
        &table_no as &(dyn tokio_postgres::types::ToSql + Sync),
    ];
    info!("Deleting item_id {} from table_no {}", item_id, table_no);
    let sql_result: SQLResult = app_state.db_executor.sql_exec(
        app_state.db_pool.clone(), 
        statement, 
        params
    ).await;
    let success_message: String = format!("Item deleted from table {}!", table_no);

    if let Err(error) = &sql_result {
        warn!("Could not delete item id {} from table {}!", item_id, table_no);
        warn!("{}", error);
    } else {
        info!("Deleted item id {} from table {}", item_id, table_no);
    }

    app_state.cache_delete(table_no).await;
    process_and_send_response(sql_result, Some(success_message))
}

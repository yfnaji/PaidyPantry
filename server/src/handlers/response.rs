use actix_web::HttpResponse;
use std::collections::HashMap;
use crate::utils::sql_executor::SQLResult;

/// Enumerates possible datatypes and purpose in response.
pub enum ResponseBody {
    /// Body sent as a hashmap with item id as keys and (String, i32) 
    /// as values for the associated item name and cooking time respectively.
    TableData(HashMap<i32, (String, i32)>),
    /// Body in the form of a message as a string.
    Message(String),
    /// Send internal server error.
    ResponseError(String)
}

/// Processes and sends the response to the client.
/// 
/// If SQL query was a SELECT query, then data will be sent to the client in the form of JSON.
/// If SQL query was a INSERT/DELETE query or the query prompted an erro then a message will be sent to the client.
/// 
/// # Arguments
/// 
/// * `sql_result` - Result of SQL query. If a SELECT query was run, it will be mapped to
///   a hashmap with item id as keys and (String, i32) as values for the associated item name and cooking time respectively,
///   otherwise None.
/// * `message` - Message to send back to client - used when an update has been made to the database.
/// 
/// Returns:
/// * An HTTP response with the results of the SELECT query.
/// * An HTTP response with the a message if the query was INSERT/DELETE query or if there is an error.
pub fn process_and_send_response(sql_result: SQLResult, success_message: Option<String>) -> HttpResponse {
    let response: ResponseBody = match sql_result {
        Ok(sql_result) => {
            match sql_result {
                Some(table_data) => ResponseBody::TableData(table_data),
                None => ResponseBody::Message(success_message.unwrap())
            }
        },
        Err(error) => {
            ResponseBody::ResponseError(error)
        }
    };
    match response {
        ResponseBody::TableData(table_data) => {
            HttpResponse::Ok().json(table_data)
        },
        ResponseBody::Message(message) => {
            HttpResponse::Ok().body(message)
        },
        ResponseBody::ResponseError(error) => {
            HttpResponse::InternalServerError().body(error)
        }
    }
}

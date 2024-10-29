use reqwest::{Client, Response, RequestBuilder};
use crate::utils::ui::pause;

/// An enum for the request types
pub enum Request {
    /// Create item request with a table number as i32 and a vector of (String, i32) tuples 
    /// representing item names and cooking times respectively 
    CreateItem(i32, Vec<(String, i32)>),
    /// Read table request with table number as i32
    ReadTable(i32),
    // Delete item request with table number and item id as i32
    DeleteItem(i32, i32),
}

/// Processes client request to server
/// 
/// Handles all types request of requests to encapsulated in the `Request` enum and
/// constructs the corresponding URL and send the HTTP request.
/// 
/// # Arguments
/// 
/// * `request` - An enum variant of `Request` representing the type of request to be sent with the relevant data to be unpacked
/// 
/// # Returns
/// - `Ok(reqwest::Response)` - Response object from the server with status code, header and body if request is successful.
/// - `Err(reqwest::Error)` - An error if there is an issue sending the request.
pub async fn process_request(
    request: Request
) -> Result<reqwest::Response, reqwest::Error> {
    
    let client: Client = Client::new();
    let mut url: String = "http://server:8080/".to_string();

    let response: RequestBuilder = match request {
        Request::DeleteItem(table_no, item_id) => {
            url.push_str(&format!("table/{}/item/{}", table_no, item_id));
            client.delete(url)
        }
        Request::ReadTable(table_no) => {
            url.push_str(&format!("table/{}", table_no));
            client.get(url)
        },
        Request::CreateItem(table_no, items_times) => {
            url.push_str(&format!("table/{}", table_no));
            client
                .post(url)
                .json(&items_times)
        },
    };
    
    response
        .send()
        .await
}

/// Process message from the server
/// 
/// # Arguments
/// 
/// * `response` - An object representing the response from the server, otherwise an error if there was an issue
///   with the response or request.
/// 
/// # Returns
/// 
/// * `Ok(reqwest::Response)` - Response object from the server with status code, header and body if request is successful.
/// * `Err(reqwest::Error)` - An error if there was an issue with processing the response.
pub async fn server_feedback(response: Result<Response, reqwest::Error>) {
    match response {
        Ok(res) => {
            match res.text().await {
                Ok(feedback) => println!("{}", feedback),
                Err(text_error) => println!("Sorry, could not read message from server:\n{}", text_error)
            }
        }
        Err(response_error) => println!("Sorry, there was an issue with this response:\n{}", response_error)
    }
    pause();
}

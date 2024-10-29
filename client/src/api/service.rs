use std::collections::HashMap;
use rand::Rng;
use reqwest::Response;
use crate::api::request::*;
use crate::utils::ui::*;

/// Send a request to the server to create items for a table.
/// 
/// The user is required to select the items they would like to
/// add to a table they have selected.
pub async fn create_items() {
    let table_no: i32 = select_table();
    let menu: Result<Vec<String>, polars::prelude::PolarsError> = get_menu();
    match menu {
        Ok(menu_items) => {
            println!("What would you like to add to table {}?\n", table_no);
            let items_times: Vec<(String, i32)> = select_menu_items(menu_items);
            let response: Result<Response, reqwest::Error> = process_request(
                Request::CreateItem(table_no, items_times)
            ).await;
            server_feedback(response).await
        },
        Err(error) => {
            println!("Sorry, there was an issue with reading the menu!\n{}", error);
            pause();
        }
    }
}

/// Returns the selected items and their corresponding times
/// 
/// Prompts the user to select the items they wish to add to a table.
/// 
/// # Arguments
/// 
/// * `menu_items` - A vector containing all the possible items that can be added to a table
/// 
/// # Returns:
/// * Vec<(String, i32)> - A list of tuples corresponding to an item name and cooking time 
///   to be added to a table
/// 
fn select_menu_items(menu_items: Vec<String>) -> Vec<(String, i32)> { 
    let mut items_times: Vec<(String, i32)> = Vec::new();
    let item_numbers: Vec<i32> = (1..=(menu_items.len() as i32)).collect();
    loop {
        display_menu(&menu_items);
        let item_no: i32 = select_item(&item_numbers);
        let time: i32 = rand::thread_rng().gen_range(5..=15);
        items_times.push((menu_items[(item_no - 1) as usize].clone(), time));
        
        println!("Would you like to add another item?\n");

        let input: String = get_user_input("\nY if yes, else press any key: \n");
        if !matches!(input.trim(), "Y" | "y") {
            break;
        }
    }
    items_times
}

/// Sends a request to the server to get data for a table.
/// 
/// If data is retrieved from the server successfully, it will then pass the data as an 
/// argument to `read_response_handler()` for further operations if necessary. Otherwise, 
/// it will simply print the feedback from the server if there is an issue.
/// 
/// # Arguments
/// 
/// * `operation` - An enum variant that defines the purpose of the reading table data. 
pub async fn read_table(operation: Operation) {
    let table_no: i32 = select_table();
    let response: Result<Response, reqwest::Error> = process_request(
        Request::ReadTable(table_no)
    ).await;

        match response {
            Ok(res) => {
                if res.status().is_success() {
                    read_response_handler(res, table_no, operation).await
                } else {
                    server_feedback(Ok(res)).await
                }
            }
            Err(e) => {println!("There was an issue trying to read that table:\n{}", e)}
        }
}

/// Process the purpose for reading the table data
/// 
/// The table items will be printed for the user to view.
/// If they selected to view a table, they will also have the additional option to look up an item 
/// from the list for additional details.
/// If they requested to delete an item, they will have to select the item they wish to
/// delete from the list.
/// If the table is empty, the user will be notified and no further action is required.
/// 
/// # Arguments
/// 
/// * `response` - Response object from the server from the GET request made earlier
/// * `table_no` - Corresponding table number
/// * `operation` - An enum variant that defines the purpose of reading the table data
async fn read_response_handler(response: Response, table_no: i32, operation: Operation) {
    match response.json::<HashMap<i32, (String, i32)>>().await {
        Ok(table_data) => {
            if table_data.is_empty() {
                println!("This table does not have items yet!");
                pause();
            } else {
                println!("Items for table {table_no}\n");
                let mut keys: Vec<i32> = table_data.keys().copied().collect();
                keys.sort();
                for (index, key) in keys.iter().enumerate() {
                    println!("{} - {}", index + 1, table_data[key].0)
                }
                match operation {
                    Operation::Read => look_up_item(table_no, &table_data, &keys),
                    Operation::Delete => delete_item(table_no, &keys).await
                }
            }
        }
        Err(e) => println!("There was an issue with the response data:\n{}", e)
    }
}

/// Optional look up details of a selected item. 
/// 
/// If the user selected to view a table, they will be presented with the option
/// to view an particular item with more detail.
/// 
/// User can select the item they wish to look up and will be prompted with the following
/// details:
/// - Table number
/// - Item name
/// - Cooking time
/// 
/// # Arguments
/// 
/// * `table_no` - Corresponding table number
/// * `table_data` - A HashMap with item ids as keys and (String, i32) tuples as values 
///   corresponding to the item name and cooking time respectively
/// * `keys` - A sorted vector containing the item ids used to view the appropriate item
fn look_up_item(table_no: i32, table_data: &HashMap<i32, (String, i32)>, keys: &[i32]) {
    let mut prompt: String = "\nQUERY an item on this table by entering its number\n".to_string();
    prompt.push_str("Otherwise, press enter\n");

    loop {
        match get_user_input(&prompt).parse::<i32>() {
            Ok(n) if (1..=keys.len() as i32).contains(&n) => {
                let item_id: &i32 = &keys[(n - 1) as usize];
                println!("\nTable Number: {}", table_no);
                println!("Item: {}", table_data[item_id].0);
                println!("Cooking time: {} minutes\n", table_data[item_id].1);
                pause();
                for (index, key) in keys.iter().enumerate() {
                    println!("{} - {}", index + 1, table_data[key].0)
                }
            },
            _ => break,
        }
    }
}

/// Sends a request to delete a selected item from a selected table.
/// 
/// User is required to select the item they wish to delete when presented
/// with the options. Once selected, a request to delete that item from 
/// the table will be sent 
async fn delete_item(table_no: i32, keys: &[i32]) {
    let item_id: i32 = select_item(keys);
    let response: Result<Response, reqwest::Error> = process_request(
        Request::DeleteItem(table_no, item_id)
    ).await;
    server_feedback(response).await
}

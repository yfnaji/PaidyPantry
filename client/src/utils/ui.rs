use std::io::{stdin, stdout, Write};
use polars::prelude::{DataFrame, CsvReadOptions, PolarsError};
use polars::io::SerReader;

/// An enum for the purpose of reading table data
pub enum Operation {
    /// Simply to read table data
    Read,
    /// See list of items to be able to delete
    Delete
}

/// Expects and processes input from the user.
/// This function is used whenever the app requires
/// input from the user such as the action to take, 
/// selecting the table number and items etc.
/// 
/// # Arguments
/// * `prompt` - The text that is displayed to the user prompting them to provide an input
/// 
/// # Returns:
/// 
/// * String - The input from the user
pub fn get_user_input(prompt: &str) -> String {
    let mut input = String::new();
    print!("{}", prompt);
    stdout().flush().unwrap();
    stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

/// Pauses the application to allow the user to read
/// prompts before moving on.
pub fn pause() {
    get_user_input("\nPress enter to continue\n");
}

/// Allows user to select a table they want to perform an action on.
/// They have the choice from tables 1 - 100.
/// 
/// # Returns:
/// 
/// * `i32`` - The table number the user wishes to perform actions on 
pub fn select_table() -> i32 { 
    loop {
        let table_no: String = get_user_input("Please select table number: 1 - 100:\n");
        match table_no.parse::<i32>() {
            Ok(n) if (1..=100).contains(&n) => {
                return n;
            },
            _ => {println!("I'm sorry, that is not a valid table number");},
        }
    }
}

/// Prompts user to select an item from a list of items on a table or the menu.
/// 
/// # Arguments
/// 
/// * `keys` - A vector of item ids either from a menu or data from a table.
/// 
/// # Returns:
/// 
/// * `i32` - The corresponding item id for an item on a table 
pub fn select_item(keys: &[i32]) -> i32 {
    loop {
        let prompt: String = "Pick an item number\n".to_string();
        match get_user_input(&prompt).parse::<i32>() {
            Ok(n) if (1..=keys.len() as i32).contains(&n) => {
                return keys[(n - 1) as usize]
            },
            _ => {
                println!("I'm sorry, that is not a valid item choice");
            },
        }
    }
}

/// Attemps to attain the items listed in menu.csv for the user to select items from.
/// 
/// # Returns
/// 
/// Returns:
/// * Ok(Vec<String>) - If read of menu.csv is successful 
/// * Err(PolarsError) - if there was an issue reading the CSV
pub fn get_menu() -> Result<Vec<String>, PolarsError> {
    let items = CsvReadOptions::default()
        .with_has_header(true)
        .try_into_reader_with_file_path(Some("menu.csv".into()));
    let dataframe: DataFrame = items?.finish()?.transpose(None, None)?;
    let mut menu: Vec<String> = Vec::new();
    for item in dataframe.iter() {
        if let Ok(item_name) = item.get(0) {
            menu.push(item_name.to_string());
        }
    }
    Ok(menu)
}

pub fn display_menu(menu: &[String]) {
    for (index, menu_item) in menu.iter().enumerate() {
        println!("{} - {menu_item}", index + 1);
    };
}

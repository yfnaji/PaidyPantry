mod api;
mod utils;

use utils::ui::{get_user_input, Operation};
use api::service::{create_items, read_table};

/// Entrypoint for client
/// 
/// User is given a choice to what action they want to perform
#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {

    println!("#==================================#");
    println!("# Welcome to the Paidy Pantry App! #");
    println!("#==================================#");

    loop {
        println!("\nWhat would you like to do?:\n");
        println!("#==========================#");
        println!("# (V)iew table items       #");
        println!("# (A)dd items to table     #");
        println!("# (D)elete item from table #");
        println!("# (Q)uit                   #");
        println!("#==========================#\n");

        let choice: String = get_user_input("Enter your choice: ");

        match choice.trim() {
            "V" | "v" => read_table(Operation::Read).await,
            "A" | "a" => create_items().await,
            "D" | "d" => read_table(Operation::Delete).await,
            "Q" | "q" => {
                println!("#=========================#");
                println!("# Thank you, and goodbye! #");
                println!("#=========================#");
                break;
            },
            _ => println!("I'm sorry, please enter a valid option\n"),
        };
    }

    Ok(())
}

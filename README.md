# Paidy Pantry

### Requirements

Only *Bash* and *Docker* are required for the applications to run.

*Cargo* is required to run the unit tests.

The server application will be listening on port `8080`, while the database will be on port `5432`.

### Instructions

cd into the project directory and follow these instructions:

1. To start the server and database, run `./start.sh`.  You may need to run `chmod +x *.sh` to allow the bash scripts to run.
2. To add a client to the session, run `./add_client.sh`. You may add multiple clients to one session.
3. To shut down the server and database, run `./stop.sh`. 

After running `./stop.sh`, you may restart the server and database by running `./start.sh` and all the data before shut down will be available.

### Unit tests

To run all unit tests, run `cargo test --manifest-path server/Cargo.toml`.

### Edits 2024/11/3
* Remove `Cargo.toml` from root directory and create individual toml files for client and server apps
* Amend data structure of cache from `IndexMap` to `HashMap` (and amend unit tests accordingly)
* Database configured to only accept table number (`table_no`) values between 1 - 100
* Include ports used by server and database in `README.md` and amend instruction for running unit tests
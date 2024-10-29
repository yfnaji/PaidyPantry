# Paidy Pantry

### Requirements

Only *Bash* and *Docker* are required for the applications to run.

*Cargo* is required to run the unit tests.

### Instructions

cd into the project directory and follow these instructions:

1. To start the server and database, run `./start.sh`.  You may need to run `chmod +x *.sh` to allow the bash scripts to run.
2. To add a client to the session, run `./add_client.sh`. You may add multiple clients to one session.
3. To shut down the server and database, run `./stop.sh`. 

After running `./stop.sh`, you may restart the server and database by running `./start.sh` and all the data before shut down will be available.

### Unit tests

To run all unit tests, run `cargo test`.

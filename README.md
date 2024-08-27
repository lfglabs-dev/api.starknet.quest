# API Starknet Quest

API for Starknet Quest Client project built in Rust

## About

API Starknet Quest provides the backend infrastructure for Starknet Quest Client, an app which helps protocols attract and retain users by creating gamified quests experiences on Starknet.

## Prerequisites

### Install Rust

To run the project without issues you need to have a Rust version >= 1.73.0. To check your rust version run the following command in a terminal.

```bash
rustc --version
```
If you don't have Rust installed, please go to the [Rust installation page](https://doc.rust-lang.org/book/ch01-01-installation.html) for further instructions.

### Install Git

Go to the [Git installation page](https://git-scm.com/downloads) and follow the instructions for your operating system to install Git.

### Install Docker

To run the database a Docker container is necessary, you need to have Docker engine version >= 1.13.0. To check your Docker engine version run the following command in a terminal.

```bash
docker --version
```
If you don't have Docker installed, please go to the [Docker installation page](https://docs.docker.com/get-started/get-docker/) for further instructions.

## Installaction Instructions

Fork the repository and clone the forked repository to your local system

```bash
git clone https://github.com/<your-user>/api.starknet.quest.git
```

## Build instructions

To build the project use the following command in a terminal

```bash
cargo build
```

The command above will run `cargo build` with the `--debug` flag, which compiles faster, includes debug symbols for easier debugging. However it produces a larger binary, for development purposes the command above is fine.

If you wish to create an optimized binary without debug information run the following command in a terminal

```bash
cargo build --release
```

## Running the project

To run the project successfully you'll need to do the following steps:
1. Deploy `db-docker-compose.yml` file to use MongoDB database
Once inside the directory of the project, you need to run the following command:
```bash
docker-compose -f db-docker-compose.yml up -d
```
The command above will create a container running the MongoDB database, however the information you add to the database isn't persistent, you'll need to modify the db-docker-compose.yml file to include a volume. For more information regarding Docker-compose files and volumes go the this [page](https://docs.docker.com/engine/storage/volumes/).

2. Create `config.toml` file using the `config.template.toml` file
To run the project successfully you need to create a `config.toml` file using the `config.template.toml` file. You can copy the file and modify the following fields accordingly:

- connection_string, if the `db-docker-compose.yml` isn't changed the connection string would be: mongodb://quests:password@localhost:27017
- secret_key, this is the secret used for the JWT token. You can change it or leave as is.
- expiry_duration, this is the expiry duration of the JWT token. You should change it according to your needs the time is stored in miliseconds.
- rpc_url, this is to interact with the blockchain you can use a public RPC such as [Lava](https://www.lavanet.xyz/get-started/starknet) or a private node provider such as [Alchemy](https://www.alchemy.com) or [Infura](https://www.infura.io)

3. Run the project 
Once the `config.toml` file is created properly, you're going to be able to run the project using the following command

```bash
cargo run
```
If you've setup everything correctly, you should see the following output

```bash
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 59.57s
     Running `target/debug/quest_server`
quest_server: starting v0.1.0
database: connected
server: listening on http://0.0.0.0:8080
```
Otherwise refer the to the Troubleshooting guide below.

## Troubleshooting


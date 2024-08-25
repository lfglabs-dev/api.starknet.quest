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

1. Deploy `db-docker-compose.yml` file to use MongoDB database
2. Create `config.toml` file using the `config.template.toml` file
3. Replace `connection_string` in `config.toml` with the proper connection string to the MongoDB database, if default credentials in `db-docker-compose.yml` file are used the connection string is: mongodb://quests:password@localhost:27017
4. Starknet RPC_URL -> can be from lava or alchemy
5. Starkscan API KEY create one

once the config.toml file is done, use cargo run to start testing


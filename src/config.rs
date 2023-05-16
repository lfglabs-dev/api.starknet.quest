use serde::Deserialize;
use starknet::core::types::FieldElement;
use std::env;
use std::fs;

macro_rules! pub_struct {
    ($name:ident {$($field:ident: $t:ty,)*}) => {
        #[derive(Clone, Deserialize)]
        pub struct $name {
            $(pub $field: $t),*
        }
    }
}

pub_struct!(Server {
    port: u16,
});

pub_struct!(Database {
    name : String,
    connection_string: String,
});

pub_struct!(NftContract {
    address: String,
    private_key: FieldElement,
});

pub_struct!(Variables {
    app_link: String,
    is_testnet: bool,
});

pub_struct!(Config {
    server : Server,
    database: Database,
    nft_contract: NftContract,
    variables: Variables,
});

pub fn load() -> Config {
    let args: Vec<String> = env::args().collect();
    let config_path = if args.len() <= 1 {
        "config.toml"
    } else {
        args.get(1).unwrap()
    };
    let file_contents = fs::read_to_string(config_path);
    if file_contents.is_err() {
        panic!("error: unable to read file with path \"{}\"", config_path);
    }

    match toml::from_str(file_contents.unwrap().as_str()) {
        Ok(loaded) => loaded,
        Err(err) => {
            panic!("error: unable to deserialize config. {}", err);
        }
    }
}

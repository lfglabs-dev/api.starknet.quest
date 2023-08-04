use serde::Serializer;
use serde::{self, Deserialize, Deserializer, Serialize};
use starknet::core::types::FieldElement;
use std::collections::HashMap;
use std::env;
use std::fs;

pub_struct!(Clone, Deserialize; Server { port: u16 });

pub_struct!(Clone, Deserialize; Database {
    name: String,
    connection_string: String,
});

pub_struct!(Clone, Deserialize; NftContract {
    address: String,
    private_key: FieldElement,
});

pub_struct!(Clone, Deserialize;  Variables {
    app_link: String,
    api_link: String,
    is_testnet: bool,
    proxy: Option<String>
});

pub_struct!(Clone, Deserialize; StarknetIdContracts {
    naming_contract: FieldElement,
    verifier_contract: FieldElement,
    identity_contract: FieldElement,
});

pub_struct!(Clone, Deserialize;  NamingContract { address: String });

pub_struct!(Clone, Deserialize;  StarknetId {
    account_id: String,
});

pub_struct!(Clone, Deserialize;  Sithswap {
    utils_contract: FieldElement,
    pairs : Vec<FieldElement>,
});

pub_struct!(Clone, Deserialize;  Quests {
    sithswap: Sithswap,
});

pub_struct!(Clone, Deserialize;  Twitter {
    oauth2_clientid: String,
    oauth2_secret: String,
});

pub_struct!(Clone, Deserialize;  Discord {
    oauth2_clientid: String,
    oauth2_secret: String,
});

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuizQuestionType {
    TextChoice,
    ImageChoice,
    Ordering,
}

impl<'de> Deserialize<'de> for QuizQuestionType {
    fn deserialize<D>(deserializer: D) -> Result<QuizQuestionType, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.to_lowercase().as_str() {
            "text_choice" => Ok(QuizQuestionType::TextChoice),
            "image_choice" => Ok(QuizQuestionType::ImageChoice),
            "ordering" => Ok(QuizQuestionType::Ordering),
            _ => Err(serde::de::Error::custom("Unexpected type")),
        }
    }
}

pub_struct!(Clone, Deserialize; QuizQuestion {
    kind: QuizQuestionType,
    layout: String,
    question: String,
    options: Vec<String>,
    correct_answer: Option<usize>,
    correct_order: Option<Vec<usize>>,
});

pub_struct!(Clone, Deserialize; Quiz {
    name: String,
    desc: String,
    questions: Vec<QuizQuestion>,
});

pub_struct!(Clone, Deserialize;  Config {
    server: Server,
    database: Database,
    nft_contract: NftContract,
    variables: Variables,
    starknetid_contracts: StarknetIdContracts,
    quests: Quests,
    twitter: Twitter,
    discord: Discord,
    quizzes: HashMap<String, Quiz>,
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

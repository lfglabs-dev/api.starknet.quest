use chrono::Utc;
use reqwest;
use serde_derive::Serialize;
use std::borrow::Cow;
use std::sync::Arc;

use crate::config::Watchtower;

// Logger structure
pub struct Logger {
    enabled: bool,
    config: Arc<Watchtower>,
    client: Arc<reqwest::Client>,
}

// Enum for log types
#[derive(Clone)]
pub enum LogType {
    Info,
    Warning,
    Severe,
}

#[derive(Serialize)]
struct LogData<'a> {
    token: &'a str,
    log: LogPayload<'a>,
}

#[derive(Serialize)]
struct LogPayload<'a> {
    app_id: &'a str,
    r#type: &'a str,
    message: Cow<'a, str>,
    timestamp: i64,
}

impl Logger {
    pub fn new(config: &Watchtower) -> Self {
        env_logger::init();
        Logger {
            enabled: config.enabled,
            config: Arc::new(config.clone()),
            client: Arc::new(reqwest::Client::new()),
        }
    }

    async fn post_log(&self, log_type: LogType, message: Cow<'static, str>) {
        let config = Arc::clone(&self.config);
        let client = Arc::clone(&self.client);

        let message_owned = message.into_owned();

        let data = LogData {
            token: &config.token,
            log: LogPayload {
                app_id: &config.app_id,
                r#type: match log_type {
                    LogType::Info => &config.types.info,
                    LogType::Warning => &config.types.warning,
                    LogType::Severe => &config.types.severe,
                },
                message: Cow::Owned(message_owned),
                timestamp: Utc::now().timestamp_millis(),
            },
        };

        let response = client.post(&config.endpoint).json(&data).send().await;

        match response {
            Ok(res) if res.status().is_success() => (),
            Ok(res) => eprintln!(
                "Failed to post log: {:?}",
                res.text().await.unwrap_or_default()
            ),
            Err(err) => eprintln!("Failed to post log: {:?}", err),
        }
    }

    pub async fn async_info<S>(&self, message: S)
    where
        S: Into<Cow<'static, str>> + std::fmt::Display + Send + 'static,
    {
        println!("INFO: {}", &message);
        if self.config.enabled {
            self.post_log(LogType::Info, message.into()).await;
        }
    }

    pub async fn async_warning<S>(&self, message: S)
    where
        S: Into<Cow<'static, str>> + std::fmt::Display + Send + 'static,
    {
        println!("WARNING: {}", &message);
        if self.config.enabled {
            self.post_log(LogType::Warning, message.into()).await;
        }
    }

    pub async fn async_severe<S>(&self, message: S)
    where
        S: Into<Cow<'static, str>> + std::fmt::Display + Send + 'static,
    {
        println!("SEVERE: {}", &message);
        if self.config.enabled {
            self.post_log(LogType::Severe, message.into()).await;
        }
    }

    pub fn info<S>(&self, message: S)
    where
        S: Into<Cow<'static, str>> + std::fmt::Display + Send + 'static,
    {
        let logger_clone = self.clone();
        tokio::spawn(async move {
            logger_clone.async_info(message).await;
        });
    }

    pub fn warning<S>(&self, message: S)
    where
        S: Into<Cow<'static, str>> + std::fmt::Display + Send + 'static,
    {
        let logger_clone = self.clone();
        tokio::spawn(async move {
            logger_clone.async_warning(message).await;
        });
    }

    pub fn severe<S>(&self, message: S)
    where
        S: Into<Cow<'static, str>> + std::fmt::Display + Send + 'static,
    {
        let logger_clone = self.clone();
        tokio::spawn(async move {
            logger_clone.async_severe(message).await;
        });
    }

    #[allow(dead_code)]
    pub fn local<S>(&self, message: S)
    where
        S: Into<Cow<'static, str>> + std::fmt::Display,
    {
        println!("{}", &message);
    }
}

impl Clone for Logger {
    fn clone(&self) -> Self {
        Logger {
            enabled: self.enabled,
            config: Arc::clone(&self.config),
            client: Arc::clone(&self.client),
        }
    }
}

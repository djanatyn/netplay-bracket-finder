use miette::{self, Diagnostic, Result};
use serde::Deserialize;
use thiserror::Error;
use tracing::{event, instrument, Level};

#[derive(Error, Debug, Diagnostic)]
#[allow(dead_code)]
enum Error {
    #[error("failed to load environment variable: {0:?}")]
    MissingVariables(envy::Error),

    #[error("request to API failed: {0:?}")]
    RequestFailed(ureq::Error),

    #[error("unknown error")]
    Unknown,
}

#[derive(Deserialize, Debug)]
struct Config {
    graphql_api_token: String,
}

#[instrument]
fn load_config() -> Result<Config> {
    match envy::from_env::<Config>() {
        Ok(config) => {
            event!(Level::INFO, "got config");
            Ok(config)
        }
        Err(e) => Err(Error::MissingVariables(e))?,
    }
}

#[instrument]
fn query_api(token: &str, query: &str) -> Result<ureq::Response> {
    let response = ureq::post("https://api.smash.gg/gql/alpha")
        .set("Authorization", format!("Bearer {token}").as_str())
        .send_json(ureq::json!({
            // skipping operationName + variables
            "query": query,
        }));

    event!(Level::INFO, ?response);

    match response {
        Ok(response) => Ok(response),
        Err(e) => Err(Error::RequestFailed(e))?,
    }
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let config = load_config()?;
    let query = include_str!("query.graphql");
    let _response = query_api(&config.graphql_api_token, query);

    // TODO: parse JSON
    // TODO: download images
    // TODO: record tournament info to database
    // TODO: output JSON for elm
    // TODO: upload JSON to google compute bucket

    Ok(())
}

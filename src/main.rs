use std::time::Duration;
use std::fmt;
use std::str;

use reqwest;
use reqwest::header::{ HeaderName, HeaderValue, HeaderMap };
use chrono::{prelude::*};
use serde::{Serialize, Deserialize};
use lambda_runtime::Context;
#[cfg(feature = "with-lambda")]
use lambda_runtime::{self, handler_fn};
#[cfg(feature = "with-lambda")]
use tokio::runtime;
#[cfg(not(feature = "with-lambda"))]
use serde_json;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum Method {
    OPTIONS,
    GET,
    POST,
    PUT,
    DELETE,
    HEAD,
    TRACE,
    CONNECT,
    PATCH,
}

impl From<Method> for reqwest::Method {
    #[inline]
    fn from(m: Method) -> reqwest::Method {
        match m {
			Method::OPTIONS => reqwest::Method::OPTIONS,
			Method::GET => reqwest::Method::GET,
			Method::POST => reqwest::Method::POST,
			Method::PUT => reqwest::Method::PUT,
			Method::DELETE => reqwest::Method::DELETE,
			Method::HEAD => reqwest::Method::HEAD,
			Method::TRACE => reqwest::Method::TRACE,
			Method::CONNECT => reqwest::Method::CONNECT,
			Method::PATCH => reqwest::Method::PATCH,
		}
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Body {
    String(String),
    Bytes(Vec<u8>),
}

impl From<Body> for reqwest::Body {
    #[inline]
    fn from(b: Body) -> reqwest::Body {
        match b {
			Body::String(str) => reqwest::Body::from(str),
			Body::Bytes(bytes) => reqwest::Body::from(bytes)
		}
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigEvent {
    pub url: Option<String>,
    pub method: Option<Method>,
	pub headers: Option<Vec<(String, String)>>,
	pub body: Option<Body>,
	pub timeout: Option<u64>,
}

#[derive(Serialize)]
struct CustomOutput {
    message: String,
}

#[derive(Debug, Serialize)]
struct CustomError {
    message: String,
}

impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for CustomError {}

type Error = Box<dyn std::error::Error + Sync + Send + 'static>;


#[cfg(feature = "with-lambda")]
#[tokio::main]
async fn main() -> Result<(), Error> {
	lambda_runtime::run(handler_fn(handler)).await?;
	Ok(())
}

#[cfg(not(feature = "with-lambda"))]
#[tokio::main]
async fn main() -> Result<(), Error> {
    let input_str = std::env::args().nth(1);
    if input_str.is_none() {
        panic!(
            "First argument must be a config provided as JSON string"
        );
    }
	let input = serde_json::from_str(&input_str.unwrap())?;
    let output = handler(input, Context::default()).await?;
    println!("{}", serde_json::to_string(&output)?);
    Ok(())
}

async fn handler(config: ConfigEvent,  _: Context) -> Result<CustomOutput, Error> {
	let config_url = config.url.unwrap();
	let config_method = config.method.unwrap_or(Method::POST);
	let config_headers = config.headers.unwrap_or(vec![]);
	let config_body = config.body.unwrap_or(Body::String("".to_string()));
	let config_timeout = config.timeout.unwrap_or(10000);

	let client = reqwest::Client::new();
	let mut headers = HeaderMap::new();
	for config_header in config_headers {
		headers.insert(HeaderName::from_bytes(config_header.0.as_bytes())?, HeaderValue::from_str(&config_header.1)?);
	}

	let builder = client
		.request(config_method.into(), config_url)
		.headers(headers)
		.body(config_body)
		.timeout(Duration::from_millis(config_timeout));

	let response = builder.send().await?;

	let response_status = response.status();
	let response_headers = response.headers().to_owned();
	let response_body : String;

	if let Ok(text) = response.text().await {
		response_body = text.to_owned();
	} else {
		response_body = "[FAILED TO PARSE BODY]".to_string();
	}

	let now = Local::now();
	
	return Ok(CustomOutput{
        message: format!(
			"Executed on {}. Status {}; Headers: {:?}; Body: {}", 
				now.format("%Y-%m-%d %H:%M:%S"), response_status, response_headers, response_body
		)
    });
}
use std::collections::HashMap;

use async_trait::async_trait;
use reqwest::header::HeaderValue;

use super::Action;
use crate::config;
use crate::{Error, PlaceholderMap, Result};

pub struct Webhook {
    url: String,
    method: reqwest::Method,
    headers: reqwest::header::HeaderMap<String>,
    body: String,
}

impl Webhook {
    fn transform_header_map(
        headers: &HashMap<String, String>,
    ) -> Result<reqwest::header::HeaderMap<String>> {
        use std::str::FromStr;
        headers
            .iter()
            .map(|(k, v)| {
                let name = reqwest::header::HeaderName::from_str(k)
                    .map_err(|x| Error(format!("Could not parse header name: {x}")))?;
                Ok((name, v.clone()))
            })
            .collect()
    }
}

impl TryFrom<&config::Action> for Webhook {
    type Error = Error;

    fn try_from(action: &config::Action) -> std::result::Result<Self, Self::Error> {
        if let config::ActionType::Webhook(web_hook) = &action.type_ {
            let mut headers = web_hook.headers.clone();
            if !headers.contains_key("User-Agent") {
                headers.insert(String::from("User-Agent"), crate::user_agent());
            }
            if web_hook.url.is_empty() {
                Err(Error(String::from("'url' cannot be empty.")))
            } else {
                Ok(Self {
                    url: web_hook.url.clone(),
                    method: reqwest::Method::from(web_hook.method),
                    headers: Self::transform_header_map(&headers)?,
                    body: web_hook.body.clone(),
                })
            }
        } else {
            panic!();
        }
    }
}

#[async_trait]
impl Action for Webhook {
    async fn trigger(&self, placeholders: PlaceholderMap) -> Result<()> {
        let url = crate::fill_placeholders(self.url.as_str(), &placeholders);
        let headers: reqwest::header::HeaderMap = self
            .headers
            .iter()
            .map(|(k, v)| {
                let value = HeaderValue::from_str(&crate::fill_placeholders(v, &placeholders))
                    .map_err(|x| Error(format!("Could not parse header value: {x}")))?;
                Ok((k.clone(), value))
            })
            .collect::<Result<_>>()?;
        let body = crate::fill_placeholders(self.body.as_str(), &placeholders);
        let client = reqwest::Client::new();
        let response = client
            .request(self.method.clone(), &url)
            .headers(headers)
            .body(body)
            .send()
            .await
            .map_err(|x| Error(format!("HTTP request failed: {x}")))?;
        let status = response.status();
        if status.is_success() {
            Ok(())
        } else {
            Err(Error(format!(
                "HTTP status code {} indicates error.",
                status.as_u16()
            )))
        }
    }
}

impl From<config::HttpMethod> for reqwest::Method {
    fn from(method: config::HttpMethod) -> Self {
        match method {
            config::HttpMethod::GET => reqwest::Method::GET,
            config::HttpMethod::POST => reqwest::Method::POST,
            config::HttpMethod::PUT => reqwest::Method::PUT,
            config::HttpMethod::DELETE => reqwest::Method::DELETE,
            config::HttpMethod::PATCH => reqwest::Method::PATCH,
        }
    }
}

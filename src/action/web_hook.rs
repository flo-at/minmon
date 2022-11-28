use std::collections::HashMap;

use super::{Action, ActionBase};
use crate::config;
use crate::{Error, PlaceholderMap, Result};
use async_trait::async_trait;

pub struct WebHook {
    action: ActionBase,
    url: String,
    method: reqwest::Method,
    headers: reqwest::header::HeaderMap<reqwest::header::HeaderValue>,
    timeout: u32,
    body: String,
}

impl WebHook {
    #[cfg(test)]
    fn new(
        name: &str,
        placeholders: PlaceholderMap,
        url: String,
        method: reqwest::Method,
        headers: reqwest::header::HeaderMap<reqwest::header::HeaderValue>,
        timeout: u32,
        body: String,
    ) -> Self {
        Self {
            action: ActionBase::new(name, placeholders),
            url,
            method,
            headers,
            timeout,
            body,
        }
    }

    fn transform_header_map(
        headers: &HashMap<String, String>,
    ) -> Result<reqwest::header::HeaderMap<reqwest::header::HeaderValue>> {
        use std::str::FromStr;
        headers
            .iter()
            .map(
                |(k, v)| -> Result<(reqwest::header::HeaderName, reqwest::header::HeaderValue)> {
                    let name = reqwest::header::HeaderName::from_str(k)
                        .map_err(|x| Error(format!("Could not parse header name: {}", x)))?;
                    let value = reqwest::header::HeaderValue::from_str(v)
                        .map_err(|x| Error(format!("Could not parse header value: {}", x)))?;
                    Ok((name, value))
                },
            )
            .collect()
    }
}

impl TryFrom<&config::Action> for WebHook {
    type Error = Error;

    fn try_from(action: &config::Action) -> std::result::Result<Self, Self::Error> {
        if let config::ActionType::WebHook(web_hook) = &action.type_ {
            Ok(Self {
                action: ActionBase::from(action),
                url: web_hook.url.clone(),
                method: reqwest::Method::from(web_hook.method),
                headers: Self::transform_header_map(&web_hook.headers)?,
                timeout: web_hook.timeout,
                body: web_hook.body.clone(),
            })
        } else {
            panic!();
        }
    }
}

#[async_trait]
impl Action for WebHook {
    async fn trigger(&self, placeholders: PlaceholderMap) -> Result<()> {
        // TODO irgendwie in Actionbase verschieben
        let placeholders = self.action.add_placeholders(placeholders)?;
        let template = text_placeholder::Template::new(self.body.as_str());
        let body = template.fill_with_hashmap(
            &placeholders
                .iter()
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .collect(),
        );
        let client = reqwest::Client::new();
        let response = client
            .request(self.method.clone(), &self.url)
            .timeout(std::time::Duration::from_secs(self.timeout.into()))
            .headers(self.headers.clone())
            .body(body)
            .send()
            .await
            .map_err(|x| Error(format!("HTTP request failed: {}", x)))?;
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

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_web_hook_ok() {
        let web_hook = WebHook::new(
            "Test WebHook",
            PlaceholderMap::new(),
            String::from("https://httpbin.org/status/200"),
            reqwest::Method::GET,
            WebHook::transform_header_map(&HashMap::new()).unwrap(),
            5,
            String::from(""),
        );
        web_hook.trigger(HashMap::new()).await.unwrap();
    }

    #[tokio::test]
    async fn test_web_hook_err() {
        let web_hook = WebHook::new(
            "Test WebHook",
            PlaceholderMap::new(),
            String::from("https://httpbin.org/status/400"),
            reqwest::Method::GET,
            WebHook::transform_header_map(&HashMap::new()).unwrap(),
            5,
            String::from(""),
        );
        assert!(web_hook.trigger(HashMap::new()).await.is_err());
    }
}

use std::collections::HashMap;

use crate::config;
use crate::placeholder::PlaceholderMap;
use async_trait::async_trait;

#[derive(Debug)]
struct Error(String, String);
impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Error while action '{}' was triggered: {}",
            self.0, self.1
        )
    }
}

#[async_trait]
pub trait Trigger: Send + Sync {
    async fn trigger(
        &self,
        placeholders: &PlaceholderMap,
    ) -> Result<(), Box<dyn std::error::Error>>;
}

pub struct WebHook {
    name: String,
    url: String,
    method: reqwest::Method,
    headers: HashMap<String, String>,
    timeout: u32,
    body: String,
}

impl WebHook {
    fn new(
        name: String,
        url: String,
        method: reqwest::Method,
        headers: HashMap<String, String>,
        timeout: u32,
        body: String,
    ) -> Self {
        Self {
            name,
            url,
            method,
            headers,
            timeout,
            body,
        }
    }
}

impl From<&config::Action> for WebHook {
    fn from(action: &config::Action) -> Self {
        let config::ActionType::WebHook(web_hook) = &action.type_;
        Self {
            name: action.name.clone(),
            url: web_hook.url.clone(),
            method: reqwest::Method::from(web_hook.method),
            headers: web_hook.headers.clone(),
            timeout: web_hook.timeout,
            body: web_hook.body.clone(),
        }
    }
}

#[async_trait]
impl Trigger for WebHook {
    async fn trigger(
        &self,
        placeholders: &PlaceholderMap,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use std::str::FromStr;
        let template = text_placeholder::Template::new(&self.body[..]);
        let body = template.fill_with_hashmap(
            &placeholders
                .iter()
                .map(|(k, v)| (&k[..], &v[..]))
                .chain(std::iter::once(("action_name", &self.name[..])))
                .collect(),
        );
        println!("Body: {}", body);
        let client = reqwest::Client::new();
        let headers: reqwest::header::HeaderMap<reqwest::header::HeaderValue> = self
            .headers
            .iter()
            .map(|(k, v)| {
                (
                    reqwest::header::HeaderName::from_str(k).unwrap(),
                    reqwest::header::HeaderValue::from_str(v).unwrap(),
                )
            })
            .collect();
        let response = client
            .request(self.method.clone(), &self.url)
            .timeout(std::time::Duration::from_secs(self.timeout.into()))
            .headers(headers)
            .body(body)
            .send()
            .await?;
        let status = response.status();
        if status.is_success() {
            Ok(())
        } else {
            Err(Box::new(Error(
                self.name.clone(),
                format!("HTTP status code {} indicates error.", status.as_u16()),
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
        let mut web_hook = WebHook::new(
            String::from("Test WebHook"),
            String::from("https://httpbin.org/status/200"),
            reqwest::Method::GET,
            HashMap::new(),
            5,
            String::from(""),
        );
        web_hook.trigger(&HashMap::new()).await.unwrap();
    }

    #[tokio::test]
    async fn test_web_hook_err() {
        let mut web_hook = WebHook::new(
            String::from("Test WebHook"),
            String::from("https://httpbin.org/status/400"),
            reqwest::Method::GET,
            HashMap::new(),
            5,
            String::from(""),
        );
        assert!(web_hook.trigger(&HashMap::new()).await.is_err());
    }
}

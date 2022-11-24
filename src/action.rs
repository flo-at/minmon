use std::collections::HashMap;

use crate::config;

type Placeholders = std::collections::HashMap<&'static str, String>;

pub trait Trigger {
    fn trigger(&mut self, placeholders: Placeholders);
}

pub struct WebHook {
    url: String,
    method: reqwest::Method,
    headers: HashMap<String, String>,
    timeout: u32,
    body: String,
}

impl From<&config::ActionWebHook> for WebHook {
    fn from(web_hook: &config::ActionWebHook) -> Self {
        Self {
            url: web_hook.url.clone(),
            method: reqwest::Method::from(web_hook.method),
            headers: web_hook.headers.clone(),
            timeout: web_hook.timeout,
            body: web_hook.body.clone(),
        }
    }
}

impl Trigger for WebHook {
    fn trigger(&mut self, placeholders: Placeholders) {
        // TODO
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

use std::collections::HashMap;

use data_encoding::BASE64;
use reqwest::{header, Client, Url};
use serde_json::Value;

pub struct BitBucket {
    pub project_url: String,
    client: Client,
}

impl BitBucket {
    pub fn new(user: &str, password: &str, host: &str, project: &str) -> Self {
        let project_url = format!(
            "https://{host}/rest/api/1.0/projects/{project}/repos",
            host = host,
            project = project
        );
        let credentials = format!("{}:{}", user, password);
        let base64 = format!("Basic {}", BASE64.encode(credentials.as_bytes()));
        let mut headers = header::HeaderMap::new();
        headers.append(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&base64).unwrap(),
        );

        let client = Client::builder()
            .default_headers(headers)
            .build()
            .expect("Failed to build HTTP client");

        Self {
            project_url,
            client,
        }
    }

    pub async fn request_repos(
        &self,
    ) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
        let url = Url::parse(&self.project_url)?;
        let current_page = self
            .client
            .get(url)
            .send()
            .await?
            .json::<HashMap<String, Value>>()
            .await?;

        let values = current_page["values"]
            .as_array()
            .expect("Can't parse repository values")
            .to_owned();

        let mut pages: Vec<Vec<Value>> = Vec::new();
        pages.push(values);

        while !current_page["isLastPage"].as_bool().unwrap_or(false) {
            let url = Url::parse(&self.project_url)?;
            let current_page = self
                .client
                .get(url)
                .query(&[("start", current_page["nextPageStart"].as_i64().unwrap())])
                .send()
                .await?
                .json::<HashMap<String, Value>>()
                .await?;

            let values = current_page["values"]
                .as_array()
                .expect("Can't parse repository values")
                .to_owned();

            pages.push(values)
        }
        Ok(pages.concat())
    }

    pub async fn request_pr_data(
        &self,
        repository: &str,
    ) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
        let path = format!("/{repo}/pull-requests", repo = repository);
        let url = Url::parse(&self.project_url)?.join(&path)?;

        let current_page = self
            .client
            .get(url)
            .send()
            .await?
            .json::<HashMap<String, Value>>()
            .await?;
        let values = current_page["values"]
            .as_array()
            .expect("Can't parse repository values")
            .to_owned();
        
        let mut pages = Vec::new();
        pages.push(values);

        while !current_page["isLastPage"].as_bool().unwrap_or(false) { 
            let path = format!("/{repo}/pull-requests", repo = repository);
            let url = Url::parse(&self.project_url)?.join(&path)?;

            let current_page = self
                .client
                .get(url)
                .query(&[("start", current_page["nextPageStart"].as_i64().unwrap())])
                .send()
                .await?
                .json::<HashMap<String, Value>>()
                .await?;

            let values = current_page["values"]
                .as_array()
                .expect("Can't parse repository values")
                .to_owned();

            pages.push(values)
        }

        Ok(pages.concat())
    }
}
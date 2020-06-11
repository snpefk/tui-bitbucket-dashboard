use std::collections::HashMap;

use data_encoding::BASE64;
use reqwest::{Client, header, Url};
use serde_json::Value;

pub struct BitBucket {
    pub project_url: String,
    client: Client,
}

impl<'a> BitBucket {
    const PR_PATH: &'a str = "/{repo}/pull-requests";

    pub fn new(user: &str, password: &str, host: &str, project: &str) -> Self {
        let project_url = format!(
            "https://{host}/rest/api/1.0/projects/{project}/repos",
            host = host,
            project = project
        );
        let credentials = format!("{}:{}", user, password);
        let base64 = format!("Basic {}", BASE64.encode(credentials.as_bytes()));
        let mut headers = header::HeaderMap::new();
        headers.append(header::AUTHORIZATION, header::HeaderValue::from_str(&base64).unwrap());

        let client =  Client::builder()
            .default_headers(headers)
            .build()
            .expect("Failed to build HTTP client");

        Self {
            project_url,
            client,
        }
    }

    pub async fn request_repos(self) {
        todo!("not implemented");
    }

    pub async fn request_pr_data(self) {
        todo!("not implemented")
    }

    async fn get_next_page(self, current_url: &str, current_page: usize) {
        todo!("not implemented")
    }

    #[tokio::main]
    pub async fn request(&self) -> Result<(), Box<dyn std::error::Error>> {
        let url = Url::parse(&self.project_url)?;
        let response = self.client.get(url)
            .send()
            .await?
            .json::<HashMap<String, Value>>()
            .await?;

        println!("{:#?}", response);

        Ok(())
    }
}

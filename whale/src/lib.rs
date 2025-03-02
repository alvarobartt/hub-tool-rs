use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::error::Error;
use url::Url;

pub struct DockerRegistry {
    pub url: Url,
    client: Client,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Repository {
    pub name: String,
    namespace: String,
    repository_type: String,
    status: usize,
    status_description: String,
    description: String,
    is_private: bool,
    star_count: usize,
    pull_count: usize,
    // TODO(start): convert to actual datetime instead
    last_updated: String,
    last_modified: String,
    date_registered: String,
    // TODO(end)
    affiliation: String,
    media_types: Vec<String>,
    content_types: Vec<String>,
    categories: Vec<String>,
    stororage_size: u64,
}

impl DockerRegistry {
    pub fn new(url: &str, token: &str) -> Result<Self, Box<dyn Error>> {
        let url = Url::parse(url)?;

        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&format!("Bearer {}", token))?,
        );

        let client = Client::builder().default_headers(headers).build()?;

        Ok(DockerRegistry { url, client })
    }

    pub async fn list_repositories(&self, org: &str) -> Result<Vec<Repository>, Box<dyn Error>> {
        let url = self.url.join(&format!("v2/repositories/{}", org))?;

        // TODO(alvarobartt): handle pagination
        let response: Value = self.client.get(url).send().await?.json().await?;
        Ok(serde_json::from_value(response["results"].clone())?)
    }

    // TODO(alvarobartt): fix here (this most likely requires pagination handling in every case)
    pub async fn list_tags(
        &self,
        org: &str,
        repository: &str,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        let url = self
            .url
            .join(&format!("v2/repositories/{}/{}/tags", org, repository))?;

        let response: Value = self.client.get(url).send().await?.json().await?;

        let mut results: Vec<String> = Vec::new();
        match response.get("results") {
            Some(rs) => {
                for r in rs.as_array().unwrap() {
                    match r.as_object().unwrap().get("images") {
                        Some(imgs) => {
                            let img = imgs.as_array().unwrap().first().unwrap().to_string();
                            results.push(img);
                        }
                        None => eprintln!("fetching images failed"),
                    }
                }
            }
            None => panic!("fetching the tags for the provided image failed"),
        }
        Ok(results)
    }
}

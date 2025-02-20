use reqwest::{header, Client};
use serde_json::Value;
use std::error::Error;
use url::Url;

pub struct DockerRegistry {
    url: Url,
    client: Client,
}

impl DockerRegistry {
    // TODO: we can provide either the registry URL for custom registries, but also the name
    // of the Docker Hub organization
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

    pub async fn list_repositories(&self) -> Result<Vec<String>, Box<dyn Error>> {
        let url = self.url.join("v2/_catalog")?;
        let response: Value = self.client.get(url).send().await?.json().await?;

        Ok(response["repositories"]
            .as_array()
            .ok_or("Invalid response format")?
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect())
    }

    pub async fn list_tags(&self, container: &str) -> Result<Vec<String>, Box<dyn Error>> {
        let url = self.url.join(&format!("v2/{}/tags/list", container))?;
        let response: Value = self.client.get(url).send().await?.json().await?;

        Ok(response["tags"]
            .as_array()
            .ok_or("Invalid response format")?
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect())
    }

    pub async fn get_manifest(
        &self,
        container: &str,
        reference: &str,
    ) -> Result<Value, Box<dyn Error>> {
        let url = self
            .url
            .join(&format!("v2/{}/manifests/{}", container, reference))?;

        let response: Value = self
            .client
            .get(url)
            .header(
                header::ACCEPT,
                "application/vnd.docker.distribution.manifest.v2+json",
            )
            .send()
            .await?
            .json()
            .await?;

        Ok(response)
    }

    #[allow(unused)]
    pub async fn get_blobs(container: &str, digest: &str) -> Result<Value, Box<dyn Error>> {
        // NOTE: if it's on a third-party registry say AWS, it will responded with a signed
        // URL and HTTP 307, so we need to capture the location and then send the request to
        // that URL
        todo!();
    }
}

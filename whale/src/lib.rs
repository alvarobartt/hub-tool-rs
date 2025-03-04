use anyhow::Context;
use chrono::{DateTime, Utc};
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use url::Url;

/// Struct that holds the client and the URL to send request to the Docker Hub
pub struct DockerHubClient {
    /// Contains the instace for the reqwest Client with the required headers and
    /// configuration if any.
    pub client: Client,

    // TODO(alvarobartt): unless custom Docker Registries are supported, the URL may not be
    // required
    /// Holds the URL for the Docker Hub (https://hub.docker.com)
    pub url: Url,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Tag {
    /// The name of the tag for a given repository in the Docker Hub
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Category {
    name: String,
    slug: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Repository {
    /// The name of the repository on the Docker Hub
    pub name: String,

    /// The namespace i.e. user or organization where the repository lives in
    namespace: String,

    /// The type of repository, can be any of "image", etc.
    repository_type: String,

    status: usize,

    status_description: String,

    // TODO: It cannot be None, but it can be empty which is practically the same, so let's handle
    // this in the future to have some consistency and use None() over Some("")
    description: String,

    is_private: bool,

    star_count: usize,

    pull_count: usize,

    last_updated: DateTime<Utc>,

    last_modified: DateTime<Utc>,

    date_registered: DateTime<Utc>,

    // TODO: same as in `description`
    affiliation: String,

    media_types: Vec<String>,

    content_types: Vec<String>,

    categories: Vec<Category>,

    /// The size of the virtual image in bytes
    storage_size: u64,
}

impl DockerHubClient {
    /// Creates a new instance of DockerHubClient with the provided authentication
    ///
    /// This method creates a new instance of the DockerHubClient with the provided token,
    /// which should have read access to the Docker Hub, to be able to call the rest of the
    /// methods within this struct. This method will configure and setup the HTTP client that
    /// will be used within the rest of the methods to send requests to the Docker Hub.
    pub fn new(token: &str) -> anyhow::Result<Self> {
        let url = Url::parse("https://hub.docker.com").context("couldn't parse docker hub url")?;

        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&format!("Bearer {}", token))
                .context("couldn't add authorization header with provided token")?,
        );

        let client = Client::builder()
            .default_headers(headers)
            .build()
            .context("couldn't build the reqwest client")?;

        Ok(Self { client, url })
    }

    /// List all the repositories under a given org or username on the Docker Hub
    ///
    /// This method lists all the repositories for a given organization or user via
    /// the `org` argument that are uploaded and publicly available on the Docker Hub.
    /// Note that if the repository is private but the provided token has access to it,
    /// then the repositories will be listed, otherwise only the public ones (if any)
    /// will be listed.
    pub async fn list_repositories(&self, org: &str) -> anyhow::Result<Vec<Repository>> {
        let url = self
            .url
            .join(&format!("v2/repositories/{}", org))
            .context("failed formatting the url with the provided org")?;

        // TODO(alvarobartt): handle pagination
        match self.client.get(url).send().await {
            Ok(response) => match response.json::<Value>().await {
                Ok(out) => Ok(serde_json::from_value(out["results"].clone())
                    .context("parsing the output json into a `Repository` struct failed")?),
                Err(e) => anyhow::bail!("failed with error {e}"),
            },
            Err(e) => anyhow::bail!("failed with error {e}"),
        }
    }

    /// List all the tags for a given repository on the Docker Hub
    ///
    /// This method expects both the organization or username via the `org`
    /// argument plus the `repository` name for the repository that the tags
    /// will be listed for.
    pub async fn list_tags(&self, org: &str, repository: &str) -> anyhow::Result<Vec<Tag>> {
        let url = self
            .url
            .join(&format!("v2/repositories/{}/{}/tags", org, repository))
            .context("failed formatting the url with the provided org and repository")?;

        // TODO(alvarobartt): handle pagination
        match self.client.get(url).send().await {
            Ok(response) => match response.json::<Value>().await {
                Ok(out) => Ok(serde_json::from_value(out["results"].clone())
                    .context("parsing the output json into a `Tag` struct failed")?),
                Err(e) => anyhow::bail!("failed with error {e}"),
            },
            Err(e) => anyhow::bail!("failed with error {e}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_repository_serde() {
        let value = json!({
          "name": "ollama",
          "namespace": "ollama",
          "repository_type": "image",
          "status": 1,
          "status_description": "active",
          "description": "The easiest way to get up and running with large language models.",
          "is_private": false,
          "star_count": 1183,
          "pull_count": 13256501,
          "last_updated": "2025-03-04T04:01:22.754331Z",
          "last_modified": "2024-10-16T13:48:34.145251Z",
          "date_registered": "2023-06-29T23:27:34.326426Z",
          "affiliation": "",
          "media_types": [
            "application/vnd.docker.container.image.v1+json",
            "application/vnd.docker.distribution.manifest.list.v2+json",
            "application/vnd.oci.image.config.v1+json",
            "application/vnd.oci.image.index.v1+json"
          ],
          "content_types": [
            "image"
          ],
          "categories": [
            {
              "name": "Machine Learning & AI",
              "slug": "machine-learning-and-ai"
            },
            {
              "name": "Developer Tools",
              "slug": "developer-tools"
            }
          ],
          "storage_size": 662988133055 as u64,
        });

        let repository = serde_json::from_value::<Repository>(value)
            .context("failed to deserialize the repository payload")
            .unwrap();

        println!("{repository:#?}");
    }

    #[tokio::test]
    async fn test_list_repositories() -> anyhow::Result<()> {
        let pat =
            std::env::var("DOCKER_PAT").context("environment variable `DOCKER_PAT` is not set")?;
        let dh =
            DockerHubClient::new(&pat).context("the docker hub client couldn't be instantiated")?;

        println!("{:#?}", dh.list_repositories("ollama").await);

        Ok(())
    }

    #[tokio::test]
    async fn test_list_tags() -> anyhow::Result<()> {
        let pat =
            std::env::var("DOCKER_PAT").context("environment variable `DOCKER_PAT` is not set")?;
        let dh =
            DockerHubClient::new(&pat).context("the docker hub client couldn't be instantiated")?;

        println!("{:#?}", dh.list_tags("ollama", "ollama").await);

        Ok(())
    }
}

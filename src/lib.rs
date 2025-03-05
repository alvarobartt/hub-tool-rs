use anyhow::Context;
use chrono::{DateTime, Utc};
use futures::future::join_all;
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
pub struct ApiResult<T> {
    count: usize,
    next: Option<String>,
    previous: Option<String>,
    results: Vec<T>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Tag {
    /// The Docker ID of the creator of the current tag
    creator: u64,

    /// The ID of the current tag on the Docker Hub
    id: u64,

    // TODO
    //   "images": [
    //     {
    //       "architecture": "amd64",
    //       "features": "",
    //       "variant": null,
    //       "digest": "sha256:96b6a4e66250499a9d87a4adf259ced7cd213e2320fb475914217f4d69abe98d",
    //       "os": "linux",
    //       "os_features": "",
    //       "os_version": null,
    //       "size": 755930694,
    //       "status": "active",
    //       "last_pulled": "2025-03-05T19:06:29.901114476Z",
    //       "last_pushed": "2024-01-16T20:54:52Z"
    //     },
    //     ...
    //  ]
    last_updated: DateTime<Utc>,
    last_updater: u64,
    last_updater_username: String,

    /// The name of the tag for a given repository in the Docker Hub
    name: String,

    repository: u64,
    full_size: u64,
    v2: bool,
    tag_status: String,
    tag_last_pulled: DateTime<Utc>,
    tag_last_pushed: DateTime<Utc>,
    media_type: String,
    content_type: String,
    digest: String,
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

        fetch::<Repository>(&self.client, &url)
            .await
            .context("fetching the provided url failed")
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

        fetch::<Tag>(&self.client, &url)
            .await
            .context("fetching the provided url failed")
    }
}

pub async fn fetch<T>(client: &Client, url: &Url) -> anyhow::Result<Vec<T>>
where
    T: for<'de> Deserialize<'de> + Send + 'static,
{
    let result = match client.get(url.clone()).send().await {
        Ok(response) => match response.json::<Value>().await {
            Ok(out) => serde_json::from_value::<ApiResult<T>>(out)
                .context("parsing the output json into an `ApiResult<T>` struct failed")?,
            Err(e) => anyhow::bail!("failed with error {e}"),
        },
        Err(e) => anyhow::bail!("failed with error {e}"),
    };

    if let Some(_) = result.next {
        let page_size = result.results.len();
        let pages = (result.count + page_size - 1) / page_size;

        // TODO: avoid spawning a bunch of tasks
        let mut tasks = Vec::new();
        for page in 2..pages {
            let new_url = url.clone();
            let new_client = client.clone();
            tasks.push(tokio::spawn(async move {
                match new_client
                    .get(new_url)
                    .query(&[("page", page), ("page_size", page_size)])
                    .send()
                    .await
                {
                    Ok(response) => match response.json::<Value>().await {
                        Ok(out) => serde_json::from_value::<ApiResult<T>>(out).context(
                            "parsing the output json into an `ApiResult<T>` struct failed",
                        ),
                        Err(e) => anyhow::bail!("failed with error {e}"),
                    },
                    Err(e) => anyhow::bail!("failed with error {e}"),
                }
            }));
        }

        let mut results = result.results;

        let futures = join_all(tasks).await;
        for future in futures {
            match future {
                Ok(Ok(result)) => {
                    results.extend(result.results);
                }
                Ok(Err(e)) => {
                    anyhow::bail!("failed to fetch: {:?}", e);
                }
                Err(e) => {
                    anyhow::bail!("failed capturing the task future: {:?}", e);
                }
            }
        }
        Ok(results)
    } else {
        Ok(result.results)
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

    #[test]
    fn test_tag_serde() {
        let value = json!({
          "creator": 14304909,
          "id": 529481097,
          "images": [
            {
              "architecture": "amd64",
              "features": "",
              "variant": null,
              "digest": "sha256:96b6a4e66250499a9d87a4adf259ced7cd213e2320fb475914217f4d69abe98d",
              "os": "linux",
              "os_features": "",
              "os_version": null,
              "size": 755930694,
              "status": "active",
              "last_pulled": "2025-03-05T07:52:00.613197154Z",
              "last_pushed": "2024-01-16T20:54:52Z"
            }
          ],
          "last_updated": "2024-01-16T20:54:55.914808Z",
          "last_updater": 14304909,
          "last_updater_username": "mxyng",
          "name": "gguf",
          "repository": 22180121,
          "full_size": 755930694,
          "v2": true,
          "tag_status": "active",
          "tag_last_pulled": "2025-03-05T07:52:00.613197154Z",
          "tag_last_pushed": "2024-01-16T20:54:55.914808Z",
          "media_type": "application/vnd.oci.image.index.v1+json",
          "content_type": "image",
          "digest": "sha256:7c49490a9e4a7ca4326e09c4b47bc525aa0a9dfc8ea0b3a30d62af23a60db712"
        });

        let tag = serde_json::from_value::<Tag>(value)
            .context("failed to deserialize the tag payload")
            .unwrap();

        println!("{tag:#?}");
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

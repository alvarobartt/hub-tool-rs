//! A (very early) asynchronous Rust library for the Docker Hub API v2
//!
//! This library exposes a client to interact with the Docker Hub via the Docker Hub API v2,
//! enabling and making it easier to get information about repositories, tags, et al. from the
//! Docker Hub via Rust; as well as to e.g. perform Hub maintenance tasks.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use anyhow::Context;
//! use hub_tool::DockerHubClient;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let client = DockerHubClient::new("dckr_pat_***")
//!         .context("couldn't initialize the docker client")?;
//!
//!     // Fetch the repositories under a given org or username on the Docker Hub
//!     let repositories = client.list_repositories("ollama")
//!         .await
//!         .context("failed while fetching the repositories")?;
//!
//!     // Fetch the tags for a given repository on the Docker Hub
//!     let tags = client.list_tags("ollama", "quantize")
//!         .await
//!         .context("failed while fetching the tags")?;
//!
//!     Ok(())
//! }
//! ```

use anyhow::Context;
use futures::future::join_all;
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use url::Url;

pub mod repositories;
pub mod tags;

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
    /// Count of the total values that are available, not the `results` length
    count: usize,

    /// The URL to query next if any, meaning that there are more results available to fetch;
    /// note that it can be null meaning that all the results have already been fetched; otherwise
    /// it contains the URL with the query values for `page` and `page_size`
    next: Option<String>,

    /// The URL to query the previous listing of results; similar to `next` but the other way
    /// around
    previous: Option<String>,

    /// A vector with the query results based on the type T
    results: Vec<T>,
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

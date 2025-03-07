use anyhow::Context;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{fetch_with_pagination, DockerHubClient};

#[derive(Serialize, Deserialize, Debug)]
pub struct Image {
    architecture: String,
    features: String,
    variant: Option<String>,
    digest: String,
    os: Option<String>,
    os_features: String,
    os_version: Option<String>,
    size: u64,
    status: String,
    last_pulled: DateTime<Utc>,
    last_pushed: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Tag {
    /// The Docker ID of the creator of the current tag
    creator: u64,

    /// The ID of the current tag on the Docker Hub
    id: u64,

    images: Vec<Image>,
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

impl DockerHubClient {
    /// List all the tags for a given repository on the Docker Hub
    ///
    /// This method expects both the organization or username via the `org`
    /// argument plus the `repository` name for the repository that the tags
    /// will be listed for.
    pub async fn list_tags(&self, org: &str, repository: &str) -> anyhow::Result<Vec<Tag>> {
        let url = self
            .url
            .join(&format!(
                "v2/namespaces/{}/repositories/{}/tags", // For some reason the endpoint `v2/repositories/{}/{}/tags` works seamlessly
                org, repository
            ))
            .context("failed formatting the url with the provided org and repository")?;

        fetch_with_pagination::<Tag>(&self.client, &url)
            .await
            .context("fetching the provided url failed")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

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
    async fn test_list_tags() -> anyhow::Result<()> {
        let pat =
            std::env::var("DOCKER_PAT").context("environment variable `DOCKER_PAT` is not set")?;
        let dh =
            DockerHubClient::new(&pat).context("the docker hub client couldn't be instantiated")?;

        println!("{:#?}", dh.list_tags("ollama", "ollama").await);

        Ok(())
    }
}

`whale` is a (still very early) Rust SDK for the Docker Hub API v2

At the moment only listing the repositories and the tags for a given org or username
is supported, the API may be unstable and subject to breaking changes.

Requirements are only `cargo add whale` and a Docker Hub Personal Access Token (PAT) to
be generated from https://app.docker.com/settings/personal-access-tokens.

```rust
use anyhow::Context;
use whale::DockerHubClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = DockerHubClient::new("dckr_pat_***");

    // Fetch the repositories under a given org or username on the Docker Hub
    let repositories = client.list_repositories("ollama")
        .await
        .context("failed while fetching the repositories")?;

    // Fetch the tags for a given repository on the Docker Hub
    let tags = client.list_repositories("ollama", "quantize")
        .await
        .context("failed while fetching the tags")?;

    Ok(())
}
```

# Docker Hub Tool

> `hub-tool` is a (still very early) Rust SDK for the Docker Hub API v2.

> [!WARNING] At the moment only listing the repositories and the tags for a
> given org or username is supported, the SDK may be unstable and subject to breaking
> changes.

Get started with `cargo add hub-tool` and a Docker Hub account, to generate a Personal
Access Token (PAT) from https://app.docker.com/settings/personal-access-tokens, to
send request to the Docker Hub API from `hub-tool`.

## What's missing?

- [ ] Support for the rest of the endpoints exposed by the Docker Hub API, read
    more about those at https://docs.docker.com/reference/api/hub/latest/
- [ ] Handle response headers for rate limiting, and make sure that requests are
    sent in batches carefully respecting those limits.
- [ ] Explore different ways of enhancing `hub-tool` to be more than a Docker Hub
    API wrapper, and add functionality on top e.g. filtering.
- [ ] Add support for custom Docker Registries (AFAIK the API differs a bit so that
    would imply most likely a different client + whatever the registry needs e.g.
    for Docker Registries in AWS the endpoints sometimes provide redirect URIs within
    the response etc.)

## Usage

```rust
use anyhow::Context;
use hub_tool::DockerHubClient;

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

## License

This project is licensed under either of the following licenses, at your option:

- [Apache License, Version 2.0](LICENSE-APACHE)
- [MIT License](LICENSE-MIT)

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this project by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.

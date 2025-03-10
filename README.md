# hub-tool

[![Crates.io](https://img.shields.io/crates/v/hub-tool.svg)](https://crates.io/crates/hub-tool)
[![Documentation](https://docs.rs/hub-tool/badge.svg)](https://docs.rs/hub-tool)

> A (very early) asynchronous Rust library for the Docker Hub API v2

> [!WARNING]
> At the moment only listing the repositories and the tags for a given org or
> username is supported, the SDK may be unstable and subject to breaking changes.
> Also due to the Docker Hub API not being really stable and not having a nice
> documentation this project is probably going to be stale until the API specification
> is clearer and works as expected.

Get started with `cargo add hub-tool` and a Docker Hub account, to generate a Personal
Access Token (PAT) from https://app.docker.com/settings/personal-access-tokens, to
send request to the Docker Hub API from `hub-tool`.

## Usage

```rust
use anyhow::Context;
use hub_tool::DockerHubClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = DockerHubClient::new("dckr_pat_***")
        .context("couldn't initialize the docker client")?;

    // Fetch the repositories under a given org or username on the Docker Hub
    let repositories = client.list_repositories("ollama")
        .await
        .context("failed while fetching the repositories")?;

    // Fetch the tags for a given repository on the Docker Hub
    let tags = client.list_tags("ollama", "quantize")
        .await
        .context("failed while fetching the tags")?;

    Ok(())
}
```
## What's missing?

- [ ] Support for the rest of the endpoints exposed by the Docker Hub API, read
    more about those at https://docs.docker.com/reference/api/hub/latest/
- [x] Handle response headers for rate limiting
- [ ] Make sure that requests are sent in batches, respecting those limits.
- [ ] Add support for custom Docker Registries (AFAIK the API differs a bit so that
    would imply most likely a different client + whatever the registry needs e.g.
    for Docker Registries in AWS the endpoints sometimes provide redirect URIs within
    the response etc.)
- [ ] Explore different ways of enhancing `hub-tool` to be more than a Docker Hub
    API wrapper, and add functionality on top e.g. filtering.

## License

This project is licensed under either of the following licenses, at your option:

- [Apache License, Version 2.0](LICENSE-APACHE)
- [MIT License](LICENSE-MIT)

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this project by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.

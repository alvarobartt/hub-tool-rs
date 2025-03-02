// use clap::Parser;
// use color_eyre::eyre::Result;
// use std::error::Error;
//
// pub mod app;
// pub use app::App;
//
// #[derive(Parser)]
// #[command(version, about, long_about = None)]
// struct Cli {
//     /// The Docker Hub URL
//     #[arg(short, long, default_value = "https://hub.docker.com")]
//     url: String,
//
//     /// The name of the organization on the Docker Hub
//     #[arg(short, long)]
//     org: String,
//
//     /// The required authentication token for the Docker Hub
//     #[arg(short, long)]
//     token: String,
// }
//
// #[tokio::main]
// async fn main() -> Result<(), Box<dyn Error>> {
//     color_eyre::install()?;
//
//     let args = Cli::parse();
//     let registry = DockerRegistry::new(&args.url, &args.token)?;
//
//     let terminal = ratatui::init();
//     let result = App::new(registry, &args.org).await.run(terminal);
//
//     ratatui::restore();
//     Ok(result?)
// }

use clap::Parser;
use color_eyre::eyre::Result;
use std::error::Error;

pub mod app;
pub use app::App;

pub mod registry;
pub use registry::DockerRegistry;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// The Docker Registry URL
    #[arg(short, long)]
    url: String,

    /// The required authentication token for the Docker Registry
    #[arg(short, long)]
    token: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    color_eyre::install()?;

    let args = Cli::parse();
    let registry = DockerRegistry::new(&args.url, &args.token)?;

    let terminal = ratatui::init();
    let result = App::new(registry).await.run(terminal);

    ratatui::restore();
    Ok(result?)
}

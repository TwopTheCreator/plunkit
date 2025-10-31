use bunja::cli::run_cli;
use env_logger::Env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    run_cli().await
}

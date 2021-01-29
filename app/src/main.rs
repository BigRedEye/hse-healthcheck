use prelude::*;

mod api;
mod config;
mod models;
mod node;
mod prelude;
mod repo;

#[actix_rt::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Node failed:\n\n{:?}", e);
    }
}

async fn run() -> Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    log::info!("Starting node");

    let config = config::Settings::new().context("Failed to parse config")?;
    let node = node::Node::new(config).await.context("Failed to create service")?;
    node.run().await.context("Service failed")?;

    Ok(())
}

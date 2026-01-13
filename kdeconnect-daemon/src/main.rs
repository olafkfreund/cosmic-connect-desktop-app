use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    info!("Starting KDE Connect daemon...");

    // TODO: Initialize daemon

    tokio::signal::ctrl_c().await?;
    info!("Shutting down...");

    Ok(())
}

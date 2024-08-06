#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    color_eyre::install().wrap_err("failed to install color_eyre error handler")?;

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .try_init()
        .wrap_err("failed to initialize logger")?;
    log_panics::init();

    let (session_tx, mut session_rx) = tokio::sync::mpsc::channel(10);
    let session_tx_clone = session_tx.clone();
    let _conn = ConnectionBuilder::session()?
        .name("com.system76.CosmicSession")?
        .serve_at(
            "/com/system76/CosmicSession",
            service::SessionService { session_tx },
        )?
        .build()
        .await?;

    loop {
        match start(session_tx_clone.clone(), &mut session_rx).await {
            Ok(Status::Exited) => {
                info!("Exited cleanly");
                break;
            }
            Ok(Status::Restarted) => {
                info!("Restarting");
            }
            Err(error) => {
                error!("Restarting after error: {:?}", error);
            }
        };
        // Drain the session channel.
        while session_rx.try_recv().is_ok() {}
    }
    Ok(())
}

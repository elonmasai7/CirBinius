use std::net::SocketAddr;
use std::sync::Arc;

use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

use cirbinius_api::config::Config;
use cirbinius_api::queue;
use cirbinius_api::router::HttpRouter;
use cirbinius_api::state;
use cirbinius_api::telemetry::{init_telemetry, startup_msg};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::from_env();
    init_telemetry();

    startup_msg(&format!("CirBinius API v{} starting", env!("CARGO_PKG_VERSION")));

    let store = state::create_store();

    // Initialize queue with workers
    queue::init_queue(store.clone(), config.clone());
    startup_msg(&format!("started {} workers", config.worker_count));

    // Periodic state snapshot
    state::spawn_snapshot_task(store.clone());

    // Create router
    let router = Arc::new(HttpRouter::new(store.clone(), config.clone()));

    let addr: SocketAddr = config.addr();
    startup_msg(&format!("listening on {addr}"));

    let listener = TcpListener::bind(addr).await?;

    loop {
        let (stream, _peer) = listener.accept().await?;
        let router = router.clone();

        tokio::spawn(async move {
            let io = TokioIo::new(stream);
            let svc = service_fn(move |req: http::Request<Incoming>| {
                let router = router.clone();
                async move {
                    let resp = router.handle(req).await;
                    Ok::<_, std::convert::Infallible>(resp)
                }
            });

            if let Err(e) = http1::Builder::new()
                .serve_connection(io, svc)
                .with_upgrades()
                .await
            {
                startup_msg(&format!("connection error: {e}"));
            }
        });
    }
}

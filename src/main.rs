mod handler;
mod load_balancer;

use env_logger::Env;
use log::{error, info};
use std::error::Error;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::mpsc::Sender;
use tokio::sync::{mpsc};
use crate::load_balancer::LoadBalancer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::Builder::from_env(Env::default()
        .default_filter_or("info"))
        .init();

    let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
    tokio::spawn(async move {
        signal_handlers(shutdown_tx).await;
    });

    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    let lb = LoadBalancer {};
    let lb = Arc::new(lb);

    loop {
        tokio::select! {
            result = listener.accept() => {
                match result {
                    Ok((socket, _)) => {
                        let lb = lb.clone();
                        tokio::spawn(async move {
                            lb.handle(socket).await;
                        });
                    }
                    Err(e) => {
                        error!("Error accepting connection {:?}", e);
                    }
                }
            }


            _ = shutdown_rx.recv() => {
                info!("Shutting down");
                break;
            }
        }
    }
    Ok(())
}


async fn signal_handlers(shutdown_tx: Sender<()>) {
    let mut sigterm = signal(SignalKind::terminate()).unwrap();
    let mut sigquit = signal(SignalKind::quit()).unwrap();

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            let _ = shutdown_tx.send(()).await;
        }
        _ = sigterm.recv() => {
            let _ = shutdown_tx.send(()).await;
        }
        _ = sigquit.recv() => {
             let _ = shutdown_tx.send(()).await;
       }
    }
}
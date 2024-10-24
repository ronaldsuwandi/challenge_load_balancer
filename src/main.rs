mod handler;
mod load_balancer;

use env_logger::Env;
use log::{error, info};
use std::error::Error;
use std::{env, fs};
use std::sync::{Arc};
use std::time::Duration;
use serde::Deserialize;
use tokio::net::TcpListener;
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::mpsc::Sender;
use tokio::sync::{mpsc};
use tokio::time::sleep;
use crate::load_balancer::{LoadBalancer, Server};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::Builder::from_env(Env::default()
        .default_filter_or("info"))
        .init();

    let conf_path = env::args().nth(1).unwrap_or_else(|| { "config.toml".to_string() });
    let servers = parse_config(&conf_path);

    let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
    tokio::spawn(async move {
        signal_handlers(shutdown_tx).await;
    });

    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    let lb = Arc::new(LoadBalancer::new(servers));

    // trigger initial healthcheck
    lb.health_check().await;

    let lb_clone = lb.clone();

    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = sleep(Duration::from_secs(5)) => {
                    info!("Executing health check...");
                    lb_clone.health_check().await;
                }
            }
        }
    });

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

fn parse_config(conf: &str) -> Vec<Server> {
    #[derive(Debug, Deserialize)]
    struct Config {
        servers: Vec<Server>,
    }

    let contents = fs::read_to_string(conf).expect("Unable to read config file");
    let config: Config = toml::from_str(&contents).expect("Unable to parse config file");

    config.servers
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

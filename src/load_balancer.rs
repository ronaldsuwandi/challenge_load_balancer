use std::io::{Read, Write};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc};
use log::{error, info, warn};
use tokio::sync::{RwLock, mpsc};
use regex::Regex;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[derive(Debug, Clone)]
pub struct Server {
    pub url: String,
    health_check_url: String,
    healthy: bool,
}

pub struct LoadBalancer {
    servers: Arc<RwLock<Vec<Server>>>,
    last_server: AtomicU32,
}

impl LoadBalancer {
    pub fn new() -> LoadBalancer {
        LoadBalancer {
            servers: Arc::new(RwLock::new(vec![
                Server {
                    url: "localhost:8081".to_string(),
                    health_check_url: "localhost:8081".to_string(),
                    healthy: true,
                },
                Server {
                    url: "localhost:8082".to_string(),
                    health_check_url: "localhost:8082".to_string(),
                    healthy: true,
                }
            ])),
            last_server: AtomicU32::new(0),

        }
    }

    pub async fn choose_server(&self) -> Option<Server> {
        let servers = self.servers.read().await; // acquire a read lock

        let healthy_servers: Vec<&Server> = servers.iter()
            .filter(|&s| s.healthy)
            .collect();

        if healthy_servers.is_empty() {
            return None;
        }

        let index = self.last_server.fetch_add(1, Ordering::SeqCst) % healthy_servers.len() as u32;
        Some(healthy_servers[index as usize].clone())
    }

    pub async fn health_check(&self) {
        let servers = self.servers.read().await;
        let (tx, mut rx) = mpsc::channel(32);

        for server in servers.iter() {
            let server = server.clone();
            let tx = tx.clone();
            tokio::spawn(async move {
                let regex_ok = Regex::new(r"^HTTP/\d\.\d 200").unwrap();
                info!("Checking {}", server.health_check_url);


                let mut targetResult = TcpStream::connect(&server.health_check_url).await;

                match targetResult {
                    Ok(mut target) => {
                        target.write_all("GET / HTTP/1.1\r\nConnection: close\r\n\r\n".as_bytes()).await.unwrap();
                        let mut buf = [0; 4096];
                        let result: (String, bool);
                        match target.read(&mut buf).await {
                            Ok(0) => {
                                info!("target stream closed");
                                result = (server.url.to_string(), false);
                            }
                            Ok(n) => {
                                info!("read from target bytes: {}", n);
                                let resp = std::str::from_utf8(&buf[0..n]).unwrap().to_string();

                                // info!("response: {}", resp.clone());

                                if regex_ok.is_match(&resp) {
                                    result = (server.url.to_string(), true);
                                } else {
                                    warn!("Server is not available {}", server.url);
                                    result = (server.url.to_string(), false);
                                }
                            }
                            Err(e) => {
                                error!("target stream read error: {:?}", e);
                                result = (server.url.to_string(), false);
                            }
                        }
                        let _ = tx.send(result).await.unwrap();
                        let _ = target.shutdown().await; // ignore if fail to shutdown socket
                    }
                    Err(e) => {
                        error!("Error connecting to {}: {}", server.health_check_url, e);
                        let _ = tx.send((server.url.to_string(), false)).await.unwrap();
                    }
                }
                drop(tx);
            });
        }

        drop(tx); // drop outer channel
        drop(servers);

        while let Some((url, healthy)) = rx.recv().await {
            let mut servers = self.servers.write().await;
            for mut server in servers.iter_mut() {
                if server.url.eq(&url) {
                    server.healthy = healthy;
                }
            }
        }
    }
}

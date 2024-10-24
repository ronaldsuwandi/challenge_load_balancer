use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc};
use log::{error, info, warn};
use tokio::sync::{RwLock, mpsc};
use regex::Regex;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Server {
    pub id: String,
    pub url: String,
    health_check_url: String,

    #[serde(default)]
    healthy: bool,
}

pub struct LoadBalancer {
    servers: Arc<RwLock<Vec<Server>>>,
    last_server: AtomicU32,
}

impl LoadBalancer {
    pub fn new(servers: Vec<Server>) -> LoadBalancer {
        LoadBalancer {
            servers: Arc::new(RwLock::new(servers)),
            last_server: AtomicU32::new(0),
        }
    }

    pub async fn choose_server(&self) -> Option<Server> {
        let servers = self.servers.read().await; // acquire a read lock

        let length = servers.len();
        let mut tries = 0;

        // try to check the next server one by one
        while tries < length {
            let index = self.last_server.fetch_add(1, Ordering::SeqCst) % length as u32;
            tries += 1;
            let server = &servers[index as usize];
            if server.healthy {
                return Some(server.clone());
            }
        }

        None
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


                let target_result = TcpStream::connect(&server.health_check_url).await;

                match target_result {
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
                                info!("response from: {}, read from target bytes: {}", server.id, n);
                                let resp = std::str::from_utf8(&buf[0..n]).unwrap().to_string();

                                // info!("response: {}", resp.clone());

                                if regex_ok.is_match(&resp) {
                                    result = (server.url.to_string(), true);
                                } else {
                                    warn!("Server is not available {}", server.id);
                                    result = (server.url.to_string(), false);
                                }
                            }
                            Err(e) => {
                                error!("target stream read error: {:?}", e);
                                result = (server.url.to_string(), false);
                            }
                        }
                        tx.send(result).await.unwrap();
                        let _ = target.shutdown().await; // ignore if fail to shutdown socket
                    }
                    Err(e) => {
                        error!("Error connecting to {}: {}", server.id, e);
                        tx.send((server.url.to_string(), false)).await.unwrap();
                    }
                }
                drop(tx);
            });
        }

        drop(tx); // drop outer channel
        drop(servers);

        while let Some((url, healthy)) = rx.recv().await {
            let mut servers = self.servers.write().await;
            for server in servers.iter_mut() {
                if server.url.eq(&url) {
                    server.healthy = healthy;
                }
            }
        }
    }
}

// mod tests {
//     use loom::sync::{Arc, RwLock};
//     use loom::thread;
//
//     #[test]
//     fn test_concurrent_health_updates() {
//         loom::model(|| {
//             let lb = Arc::new(LoadBalancer::new());
//
//             let lb1 = Arc::clone(&lb);
//             let t1 = thread::spawn(move || {
//                 lb1.update_health("localhost:8081", false);
//             });
//
//             let lb2 = Arc::clone(&lb);
//             let t2 = thread::spawn(move || {
//                 let _ = lb2.get_healthy_server();
//             });
//
//             t1.join().unwrap();
//             t2.join().unwrap();
//         });
//     }
// }
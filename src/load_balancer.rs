use std::io::{Read, Write};
use std::net::{Shutdown, TcpStream};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, RwLock};
use log::{error, info, warn};
use regex::Regex;

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

    pub fn choose_server(&self) -> Option<Server> {
        let servers = self.servers.read().unwrap(); // acquire a read lock

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
        let mut servers = self.servers.write().unwrap();
        let regex_ok = Regex::new(r"^HTTP/\d\.\d 200").unwrap();
        for server in servers.iter_mut() {
            info!("Checking {}", server.health_check_url);
            let mut target = TcpStream::connect(&server.health_check_url).unwrap();
            target.write_all("GET / HTTP/1.1\r\nConnection: close\r\n\r\n".as_bytes()).unwrap();
            let mut buf = [0; 4096];
            match target.read(&mut buf) {
                Ok(0) => {
                    info!("target stream closed");
                }
                Ok(n) => {
                    info!("read from target bytes: {}", n);
                    let resp = std::str::from_utf8(&buf[0..n]).unwrap().to_string();

                    info!("response: {}", resp.clone());

                    if regex_ok.is_match(&resp) {
                        server.healthy = true;
                    } else {
                        warn!("Server is not available {}", server.url);
                        server.healthy = false;
                    }
                }
                Err(e) => {
                    error!("target stream read error: {:?}", e);
                    server.healthy = false
                }
            }
            let _ = target.shutdown(Shutdown::Both); // ignore if fail to shutdown socket
        }
    }
}

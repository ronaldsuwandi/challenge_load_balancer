use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, RwLock};

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
}

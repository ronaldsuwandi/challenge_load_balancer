pub struct Server {
    pub(crate) url: String,
    health_check_url: String,
    healthy: bool,
}

pub struct LoadBalancer {
    servers: Vec<Server>,
    last_server: u32,
}

impl LoadBalancer {
    pub fn new() -> LoadBalancer {
        LoadBalancer {
            servers: vec![
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
            ],
            last_server: 0,

        }
    }

    pub fn choose_server(&mut self) -> &Server {
        self.last_server = (self.last_server + 1) % self.servers.len() as u32;
        &self.servers[self.last_server as usize]
    }
}

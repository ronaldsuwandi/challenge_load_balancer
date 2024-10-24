use std::error::Error;
use tokio::net::{TcpStream};
use log::{debug, error, info, warn};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::load_balancer::LoadBalancer;

impl LoadBalancer {
    pub async fn handle(&self, mut socket: TcpStream) -> Result<(), Box<dyn Error>> {
        if let Some(target_server) = self.choose_server().await {
            let mut req_buffer = [0; 4096];
            // split socket read/write so it can be used by both async functions
            let (mut socket_rd, mut socket_wr) = socket.split();

            let mut target_stream = TcpStream::connect(&target_server.url).await.unwrap();
            let (mut target_rd, mut target_wr) = target_stream.split();

            let client_to_target = async {
                // loop here so we can stream the input (large input)
                loop {
                    // read input
                    match socket_rd.read(&mut req_buffer).await {
                        Ok(0) => {
                            debug!("Input stream closed");
                            break;
                        }
                        Ok(n) => {
                            target_wr.write_all(&req_buffer[0..n]).await?;
                        }
                        Err(e) => {
                            return Err(e);
                        }
                    };
                }
                let _ = target_wr.shutdown().await;
                Ok(())
            };

            let mut resp_buffer = [0; 4096];
            let target_to_client = async {
                // loop here so we can stream the output (for large output)
                loop {
                    match target_rd.read(&mut resp_buffer).await {
                        Ok(0) => {
                            debug!("Target stream closed");
                            break;
                        }
                        Ok(n) => {
                            info!("Read from server {}, target bytes: {}", target_server.id, n);
                            socket_wr.write(&resp_buffer[0..n]).await?;
                        }
                        Err(e) => {
                            return Err(e);
                        }
                    }
                }
                let _ = socket_wr.shutdown().await;
                Ok(())
            };

            if let Err(e) = tokio::try_join!(client_to_target, target_to_client) {
                error!("Error: {:?}", e);
                return Err(e.into());
            }
        } else {
            warn!("No healthy server");
            let response = "HTTP/1.1 502 Bad Gateway\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
            socket.write_all(response.as_bytes()).await?;
            let _ = socket.shutdown().await;
        }
        Ok(())
    }
}

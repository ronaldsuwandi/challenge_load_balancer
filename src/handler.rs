use tokio::net::{TcpStream};
use log::{error, info};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::load_balancer::LoadBalancer;

impl LoadBalancer {
    pub async fn handle(&mut self, mut socket: TcpStream) {
        let mut req_buffer = [0; 4096];
        let target_server = self.choose_server();
        let mut target_stream = TcpStream::connect(target_server.url.clone()).await.unwrap();


        // split socket read/write so it can be used by both async functions
        let (mut socket_rd, mut socket_wr) = socket.split();
        let (mut target_rd, mut target_wr) = target_stream.split();

        let client_to_target = async {
            // loop here so we can stream the input (large input)
            loop {
                // read input
                match socket_rd.read(&mut req_buffer).await {
                    Ok(0) => {
                        info!("socket closed");
                        break;
                    }
                    Ok(n) => {
                        info!("read bytes: {}", n);
                        let req = std::str::from_utf8(&req_buffer[0..n]).unwrap().to_string();
                        info!("req: {:?}", req);
                        target_wr.write_all(&req_buffer[0..n]).await.unwrap();
                    }
                    Err(e) => {
                        error!("socket read error: {:?}", e);
                        return;
                    }
                };
            }
            if let Err(e) = target_wr.shutdown().await {
                error!("Error shutting down target writer: {:?}", e);
            }
        };

        let mut resp_buffer = [0; 4096];
        let target_to_client = async {
            // loop here so we can stream the output (for large output)
            loop {
                match target_rd.read(&mut resp_buffer).await {
                    Ok(0) => {
                        info!("target stream closed");
                        break;
                    }
                    Ok(n) => {
                        info!("read from target bytes: {}", n);
                        socket_wr.write(&resp_buffer[0..n]).await.unwrap();
                    }
                    Err(e) => {
                        error!("target stream read error: {:?}", e);
                        return;
                    }
                }
            }
        };

        tokio::join!(client_to_target, target_to_client);
    }
}
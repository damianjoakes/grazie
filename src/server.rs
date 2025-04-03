use tokio::net::{TcpListener, ToSocketAddrs};

pub struct HttpServer {
    listener: TcpListener,
}

impl HttpServer {
    pub async fn new<A: ToSocketAddrs>(host: A) -> std::io::Result<HttpServer> {
        let listener = TcpListener::bind(host).await?;

        Ok(HttpServer {
            listener,
        })
    }

    /// Runs the HTTP server.
    pub async fn run(&self) -> std::io::Result<()> {
        unimplemented!()
        // loop {
        //     let (mut socket, _) = self.listener.accept().await?;
        //
        //     tokio::spawn(async move {
        //         let mut buf = vec![0; 1024];
        //
        //         loop {
        //             match socket.read(&mut buf).await {
        //                 // Return value of `Ok(0)` signifies that the remote has
        //                 // closed
        //                 Ok(0) => return,
        //                 Ok(n) => {
        //                     // Copy the data back to socket
        //                     if socket.write_all("HTTP/1.1 200 OK\r\n\r\n".as_bytes()).await.is_err() {
        //                         socket.shutdown().await.unwrap();
        //                         // Unexpected socket error. There isn't much we can
        //                         // do here so just stop processing.
        //                         return;
        //                     }
        //
        //                     socket.shutdown().await.unwrap();
        //                 }
        //                 Err(_) => {
        //                     // Unexpected socket error. There isn't much we can do
        //                     // here so just stop processing.
        //                     return;
        //                 }
        //             }
        //         }
        //     });
        // }
    }
}

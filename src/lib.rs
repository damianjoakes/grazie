#[cfg(test)]
mod tests;

pub mod core;
mod server;


pub mod http {
    pub use hyper::StatusCode;
    pub use hyper::Request as HttpRequest;
    pub use hyper::Response as HttpResponse;
}

pub use crate::server::HttpServer;
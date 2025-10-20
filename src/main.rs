use krb5proxy::config;
use tokio::net::TcpListener;
use std::net::SocketAddr;
use std::sync::Arc;
use krb5proxy::logging::Logger;


use bytes::Bytes;
use http_body_util::combinators::BoxBody;
use hyper::service::service_fn;
use hyper::{Request, Response};

use krb5proxy::proxy::{self, RequestContext, RequestState};
use krb5proxy::args::Args;
use krb5proxy::config::Config;


#[path = "../benches/support/mod.rs"]
mod support;
use support::TokioIo;

type ServerBuilder = hyper::server::conn::http1::Builder;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

    let args = Args::new();
    let log = build_logger(&args).await;
    let config = std::sync::Arc::new(config::Config::builder()
        .with_proxy_ip(args.get_proxy_ip())
        .with_proxy_port(args.get_proxy_port())
        .with_kerberos_service(args.get_kerberos_service())
        .with_listen_address(args.get_listen())
        .with_logger(log.clone())
        .build());

    let ip: std::net::IpAddr = config.get_listen_ip()
        .parse()
        .expect("Invalid IP address");

    let port = config.get_listen_port();
    let addr = SocketAddr::new(ip, port);

    let listener = TcpListener::bind(addr).await?;

    log.info("Listening on http://".to_string() + &config.get_listen_ip() + ":" + &config.get_listen_port().to_string()).await;

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let config = config.clone();
        let log = log.clone();

        tokio::task::spawn(async move {
            if let Err(err) = ServerBuilder::new()
                .preserve_header_case(true)
                .title_case_headers(true)
                .serve_connection(io, service_fn(|req| request_machine(req, config.clone())))
                .with_upgrades()
                .await
            {
                log.error(format!("Failed to serve connection: {:?}", err)).await;
            }
        });
    }


}


async fn request_machine(req: Request<hyper::body::Incoming>, config: std::sync::Arc<Config>) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {

    let mut state = RequestState::WaitingForRequest;
    let mut context = RequestContext::new(req).await;

loop {
    state = state.next(&mut context, &config).await;

    if matches!(state, RequestState::Closing) {
        break; // exit the loop to return response
        }
    }

    let resp_to_client = match  context.original_response.take() {
        Some(response) => {
            // println!("response: {:?}", response);
            response
        }
        None => {
            println!("no response");
            Response::new(proxy::empty())
        }
    };

    Ok(resp_to_client)
}

async fn build_logger(args: &Args) -> Arc<Logger> {
    Arc::new(Logger::build(args.get_log().as_str()))
}
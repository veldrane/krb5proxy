use tokio::net::TcpListener;
use std::net::SocketAddr;


use bytes::Bytes;
use http_body_util::combinators::BoxBody;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper::body::Body;

use krb5proxy::proxy::{self, RequestContext, RequestState};


#[path = "../benches/support/mod.rs"]
mod support;
use support::TokioIo;

type ServerBuilder = hyper::server::conn::http1::Builder;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

    let addr = SocketAddr::from(([10, 4, 0, 21], 8080));

    let listener = TcpListener::bind(addr).await?;
    println!("Listening on http://{}", addr);

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        tokio::task::spawn(async move {
            if let Err(err) = ServerBuilder::new()
                .preserve_header_case(true)
                .title_case_headers(true)
                .serve_connection(io, service_fn(request_machine))
                .with_upgrades()
                .await
            {
                println!("Failed to serve connection: {:?}", err);
            }
        });
    }


}


async fn request_machine(req: Request<hyper::body::Incoming>) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {

    let mut state = RequestState::WaitingForRequest;
    let mut context = RequestContext::new(req).await;

loop {
    
    state = state.next(&mut context).await;

    if matches!(state, RequestState::Closing) {
        break; // konec práce
        }
    }

    println!("state: {:?}", context);

    let resp_to_client = match  context.original_response.take() {
        Some(response) => {
            println!("response: {:?}", response);
            response
        }
        None => {
            println!("no response");
            Response::new(proxy::empty())
        }
    };

    //Ok(response.map(|b| b.boxed()))

    //let mut resp_to_client = Response::new(empty());
    //*resp_to_client.status_mut() = StatusCode::OK;

    Ok(resp_to_client)

}

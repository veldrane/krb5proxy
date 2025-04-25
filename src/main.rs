use http::HeaderValue;
use libgssapi::oid::{OidSet, GSS_NT_HOSTBASED_SERVICE, GSS_MECH_KRB5};
use krb5proxy::krb5::setup_client_ctx;
use krb5proxy::upstream::connect;
use libgssapi::name::Name;
use base64::engine::Engine;

use tokio::net::{TcpListener, TcpStream};
use std::net::SocketAddr;


use bytes::Bytes;
use http_body_util::{combinators::BoxBody, BodyExt, Empty, Full};
use hyper::service::service_fn;
use hyper::upgrade::Upgraded;
use hyper::{Method, Request, Response};

#[path = "../benches/support/mod.rs"]
mod support;
use support::TokioIo;


type ClientBuilder = hyper::client::conn::http1::Builder;
type ServerBuilder = hyper::server::conn::http1::Builder;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));

    let listener = TcpListener::bind(addr).await?;
    println!("Listening on http://{}", addr);

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        tokio::task::spawn(async move {
            if let Err(err) = ServerBuilder::new()
                .preserve_header_case(true)
                .title_case_headers(true)
                .serve_connection(io, service_fn(proxy))
                .with_upgrades()
                .await
            {
                println!("Failed to serve connection: {:?}", err);
            }
        });
    }


}


async fn proxy(
    mut req: Request<hyper::body::Incoming>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    println!("req: {:?}", req);

    let apreq :HeaderValue = get_apreq_header().await.unwrap().parse().unwrap();

    req.headers_mut().insert(
        hyper::header::PROXY_AUTHORIZATION,
        apreq,
    );

        let proxy_host = format!("10.4.0.254");
        let proxy_port = 8080;

        let stream = TcpStream::connect((proxy_host, proxy_port)).await.unwrap();
        let io = TokioIo::new(stream);

        let (mut sender, conn) = ClientBuilder::new()
            .preserve_header_case(true)
            .title_case_headers(true)
            .handshake(io)
            .await?;

        tokio::task::spawn(async move {
            if let Err(err) = conn.await {
                println!("Connection failed: {:?}", err);
            }
        });

    if Method::CONNECT == req.method() {
        // Received an HTTP request like:
        // ```
        // CONNECT www.domain.com:443 HTTP/1.1
        // Host: www.domain.com:443
        // Proxy-Connection: Keep-Alive
        // ```
        //
        // When HTTP method is CONNECT we should return an empty body
        // then we can eventually upgrade the connection and talk a new protocol.
        //
        // Note: only after client received an empty body with STATUS_OK can the
        // connection be upgraded, so we can't return a response inside
        // `on_upgrade` future.
        if let Some(addr) = host_addr(req.uri()) {
            tokio::task::spawn(async move {
                match hyper::upgrade::on(req).await {
                    Ok(upgraded) => {
                        if let Err(e) = tunnel(upgraded, addr).await {
                            eprintln!("server io error: {}", e);
                        };
                    }
                    Err(e) => eprintln!("upgrade error: {}", e),
                }
            });

            Ok(Response::new(empty()))
        } else {
            eprintln!("CONNECT host is not socket addr: {:?}", req.uri());
            let mut resp = Response::new(full("CONNECT must be to a socket address"));
            *resp.status_mut() = http::StatusCode::BAD_REQUEST;

            Ok(resp)
        }
    } else {
        //let host = req.uri().host().expect("uri has no host");
        //let port = req.uri().port_u16().unwrap_or(80);
        

        let resp = sender.send_request(req).await?;
        Ok(resp.map(|b| b.boxed()))
    }
}

fn host_addr(uri: &http::Uri) -> Option<String> {
    //uri.authority().map(|auth| auth.to_string())
    "10.4.0.254:8080".parse::<SocketAddr>().ok().map(|addr| addr.to_string())
}

fn empty() -> BoxBody<Bytes, hyper::Error> {
    Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed()
}

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

// Create a TCP connection to host:port, build a tunnel between the connection and
// the upgraded connection
async fn tunnel(upgraded: Upgraded, addr: String) -> std::io::Result<()> {
    // Connect to remote server
    let mut server = TcpStream::connect(addr).await?;
    let mut upgraded = TokioIo::new(upgraded);

    // Proxying data
    let (from_client, from_server) =
        tokio::io::copy_bidirectional(&mut upgraded, &mut server).await?;

    // Print message when done
    println!(
        "client wrote {} bytes and received {} bytes",
        from_client, from_server
    );

    Ok(())
}

async fn get_apreq_header() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {

        let mut desired_mechs = OidSet::new()?;
    desired_mechs.add(&GSS_MECH_KRB5)?;


    let cname = Name::new(b"HTTP@proxy.class.syscallx86.com", Some(&GSS_NT_HOSTBASED_SERVICE)).expect("Failed to create name");

    let mut client_ctx = setup_client_ctx(cname, &desired_mechs);

    match &client_ctx {
        Ok(ctx) => println!("Client context created successfully: {:#?}", &ctx),
        Err(e) => println!("Failed to create client context: {:#?}", e),
        
    }

    println!("Client context: {:#?}", client_ctx);

    let gss_buffer = match client_ctx.as_mut().unwrap().step(None, None) {
        Ok(t) => t,
        Err(e) => None
    };


    let ap_req = match gss_buffer {
        Some(buf) => {
            let b64 = base64::engine::general_purpose::STANDARD.encode(buf.as_ref());
            format!("Negotiate {}", b64)
        },
        None => panic!("Failed to get AP-REQ token"),
    };

    return  Ok(ap_req.to_string());

}


async fn auth_and_connect() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    
    let mut desired_mechs = OidSet::new()?;
    desired_mechs.add(&GSS_MECH_KRB5)?;


    let cname = Name::new(b"HTTP@proxy.class.syscallx86.com", Some(&GSS_NT_HOSTBASED_SERVICE)).expect("Failed to create name");

    let mut client_ctx = setup_client_ctx(cname, &desired_mechs);

    match &client_ctx {
        Ok(ctx) => println!("Client context created successfully: {:#?}", &ctx),
        Err(e) => println!("Failed to create client context: {:#?}", e),
        
    }

    println!("Client context: {:#?}", client_ctx);

    let gss_buffer = match client_ctx.as_mut().unwrap().step(None, None) {
        Ok(t) => t,
        Err(e) => None
    };


    let ap_req = match gss_buffer {
        Some(buf) => {
            let b64 = base64::engine::general_purpose::STANDARD.encode(buf.as_ref());
            format!("Negotiate {}", b64)
        },
        None => panic!("Failed to get AP-REQ token"),
    };

    println!("client ctx: {:#?}", client_ctx);

    connect(&ap_req).await?;

    Ok(())
} 
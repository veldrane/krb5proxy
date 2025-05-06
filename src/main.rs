use http::HeaderValue;
use libgssapi::oid::{OidSet, GSS_NT_HOSTBASED_SERVICE, GSS_MECH_KRB5};
use krb5proxy::krb5::setup_client_ctx;
use krb5proxy::upstream::connect;
use libgssapi::name::Name;
use base64::engine::Engine;

use tokio::io::{copy_bidirectional, AsyncWriteExt, AsyncReadExt, ErrorKind};
use tokio::net::{TcpListener, TcpStream};
use std::net::SocketAddr;


use bytes::Bytes;
use http_body_util::{combinators::BoxBody, BodyExt, Empty, Full};
use hyper::service::service_fn;
use hyper::upgrade::Upgraded;
use hyper::body::Body;
use hyper::{Method, Request, Response};

use krb5proxy::proxy::{RequestState, RequestContext};

use http::StatusCode;

#[path = "../benches/support/mod.rs"]
mod support;
use support::TokioIo;


type ClientBuilder = hyper::client::conn::http1::Builder;
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

    let resp_to_client = context.original_response.take().unwrap();

    //Ok(response.map(|b| b.boxed()))

    //let mut resp_to_client = Response::new(empty());
    //*resp_to_client.status_mut() = StatusCode::OK;

    Ok(resp_to_client)

}


async fn proxy(
    mut req: Request<hyper::body::Incoming>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    println!("req: {:?}", req);

    let apreq :HeaderValue = get_apreq_header().await.unwrap().parse().unwrap();

    //req.headers_mut().insert(
    //    hyper::header::PROXY_AUTHORIZATION,
    //    apreq,
    // ) ;

    let proxy_host = format!("10.4.0.254");
    let proxy_port = 8080;

    let mut proxy_stream = TcpStream::connect((proxy_host, proxy_port)).await.unwrap();
    // let io = TokioIo::new(stream);


    if Method::CONNECT == req.method() {

        //let proxy_request = Request::builder()
        //    .method(Method::CONNECT)
        //    .uri(req.uri())
        //    .header(hyper::header::HOST, req.uri().to_string())
        //    .header(hyper::header::PROXY_AUTHORIZATION, apreq)
        //    .body(Empty::<Bytes>::new())
        //    .unwrap();

        // let proxy_resp = proxy_sender.send_request(proxy_request).await?;

        //if proxy_resp.status() != http::StatusCode::OK {
        //    return Ok(proxy_resp.map(|b| b.boxed()));
        //}

        let proxy_string= apreq.to_str().unwrap();

        let connect_req = format!(
            "CONNECT {} HTTP/1.1\r\nHost: {}\r\nProxy-Authorization: {}\r\n\r\n",
            req.uri(), "email.cz", proxy_string
        );

        proxy_stream.write_all(connect_req.as_bytes()).await.unwrap();
    
        // 3. Přečti odpověď a ověř 200 OK
        let mut buf = [0u8; 4096];

        let n = proxy_stream.read(&mut buf).await.unwrap();

        println!("buf: {:?}", &buf[..n]);

        

        let proxy_response = String::from_utf8_lossy(&buf[..n]);

        if !proxy_response.starts_with("HTTP/1.1 200") {
            let client_response = Response::builder()
                .status(StatusCode::FORBIDDEN) // tady změníš kód
                .body(full("Forbidden"))
                .unwrap();
            return Ok(client_response);
        }

        println!("proxy_response: {:?}", proxy_response);


        tokio::task::spawn(async move {
            let mut upgraded_client = TokioIo::new(hyper::upgrade::on(req).await.unwrap());
            match tokio::io::copy_bidirectional(&mut upgraded_client, &mut proxy_stream).await {
                Ok((from_client, from_server)) => {
                    println!(
                        "client wrote {} bytes and received {} bytes",
                        from_client, from_server
                    );
                }
                Err(e) if e.kind() == ErrorKind::NotConnected => {
                    // benigní stav: peer poslal FIN a pak shutdown; můžeme to ignorovat
                    println!("Tunel ENOTCONN při shutdown, ignoruji");
                }
                Err(e) => {
                    println!("Tunel selhal: {:?}", e);
                }
            }
        });

        //let mut upgraded_proxy = TokioIo::new(proxy_stream);
        //let mut upgraded_client = TokioIo::new(hyper::upgrade::on(req).await?);
        //println!("upgraded_client: {:?}", upgraded_client);
        //tokio::io::copy_bidirectional(&mut upgraded_client, &mut proxy_stream).await.unwrap();

        //println!("upgraded_client2: {:?}", upgraded_client);


        let mut resp_to_client = Response::new(empty());
        *resp_to_client.status_mut() = StatusCode::OK;




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

        Ok(resp_to_client)

    } else {

        let io = TokioIo::new(proxy_stream);

        let (mut proxy_sender, proxy_conn) = ClientBuilder::new()
            .preserve_header_case(true)
            .title_case_headers(true)
            .handshake(io)
            .await?;

        tokio::task::spawn(async move {
            if let Err(err) = proxy_conn.await {
                println!("Connection failed: {:?}", err);
            }
        });
        
        req.headers_mut().insert(
            hyper::header::PROXY_AUTHORIZATION,
            apreq,
        );

        let (parts, _body) = req.into_parts();
        let new_req = http::Request::from_parts(parts, Empty::<Bytes>::new());
        let resp = proxy_sender.send_request(new_req).await?;
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
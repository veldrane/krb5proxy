
use http_body_util::Empty;
use hyper::Request;
use hyper::body::Bytes;
use hyper_util::rt::TokioIo;
use tokio::net::TcpStream;
use hyper::client::conn::http1::Builder as HyperBuilder;

//const TICKET : &str = "Negotiate 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";


pub async fn connect (ap_req: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let url = "http://email.cz".parse::<hyper::Uri>()?;

    // Get the host and the port
    //let host = url.host().expect("uri has no host");
    // let port = url.port_u16().unwrap_or(8080);
    
    let address = format!("10.4.0.254:8080");
    
    // Open a TCP connection to the remote host
    let stream = TcpStream::connect(address).await?;
    
    // Use an adapter to access something implementing `tokio::io` traits as if they implement
    // `hyper::rt` IO traits.
    let io = TokioIo::new(stream);
    
    // Create the Hyper client
    // let (mut sender, conn) = hyper::client::conn::http1::handshake::<TokioIo<tokio::net::TcpStream>, Empty<Bytes>>(io).await?;
    
    let mut builder = HyperBuilder::new();
    builder
        .preserve_header_case(true)
        .title_case_headers(true);


    let (mut sender, conn) = builder.handshake::<_, Empty<Bytes>>(io).await?;

    // Spawn a task to poll the connection, driving the HTTP state
    tokio::task::spawn(async move {
        if let Err(err) = conn.await {
            println!("Connection failed: {:?}", err);
        }
    });


    let authority = url.authority().unwrap().clone();

    // Create an HTTP request with an empty body and a HOST header
    let req = Request::builder()
            .uri(url)
            .header(hyper::header::HOST, authority.as_str())
            .header(hyper::header::PROXY_AUTHORIZATION, &ap_req.to_string())
            .header(hyper::header::USER_AGENT, "hyper/0.15")
            .header("Proxy-Connection", "Keep-Alive")
            .body(Empty::<Bytes>::new())?;

    // Await the response...
    let res = sender.send_request(req).await?;

    println!("Response status: {}", res.status());

    Ok(())
}
use libgssapi::name::Name;
use libgssapi::oid::{OidSet, GSS_NT_HOSTBASED_SERVICE, GSS_MECH_KRB5};
use libgssapi::credential::{Cred, CredUsage};
use libgssapi::context::{ClientCtx, CtxFlags};
use base64::engine::Engine;
use tokio::io::{AsyncWriteExt, AsyncReadExt, ErrorKind};
use tokio::net::TcpStream;
use hyper::{Request, Response};
use bytes::Bytes;
use hyper::Error;

use http_body_util::{combinators::BoxBody, BodyExt, Empty};
use http::StatusCode;

type ClientBuilder = hyper::client::conn::http1::Builder;

#[path = "../benches/support/mod.rs"]
mod support;
use support::TokioIo;

#[derive(Debug)]
pub enum RequestState {
    WaitingForRequest,
    GettingTicket,
    ConnectingToProxy,
    Tunelling,
    Forwarding,
    Closing
}

#[derive(Debug)]
enum StateErrors {
    NoError,
    MissingTgt,
    FailedGettingServiceTicket,
    FailedProxyConnection,
    FaildedProxyAuth,
    FailedTunnel,
}

#[derive(Debug)]
pub struct RequestContext {
    kerberos_service: Vec<u8>,
    proxy_ip: String,
    proxy_port: u16,
    ap_req: Option<String>,
    last_state: RequestState,
    last_error: StateErrors,
    error_message: Option<String>,
    original_request: Option<Request<hyper::body::Incoming>>,
    pub original_response: Option<Response<BoxBody<Bytes, Error>>>,
}

impl RequestState {

pub async fn next(self, ctx: &mut RequestContext) -> RequestState {
    match self {
        RequestState::WaitingForRequest => ctx.handle_waiting_for_request().await,
        RequestState::GettingTicket => ctx.handle_getting_ticket().await,
        RequestState::ConnectingToProxy => ctx.handle_connecting_to_proxy().await,
        RequestState::Tunelling => ctx.handle_tunelling().await,
        RequestState::Forwarding => ctx.handle_forwarding().await,
        RequestState::Closing => ctx.handle_closing().await,
        }
    }
}

impl RequestContext {
    pub async fn new(req: Request<hyper::body::Incoming>) -> Self {
        RequestContext {
            kerberos_service: b"HTTP@proxy.class.syscallx86.com".to_vec(),
            proxy_ip: "10.4.0.254".to_string(),
            proxy_port: 8080,
            ap_req: None,
            last_state: RequestState::WaitingForRequest,
            last_error: StateErrors::NoError,
            error_message: None,
            original_request: Some(req),
            original_response: None,
        }
    }

    pub async fn handle_waiting_for_request(&mut self) -> RequestState {

        // Handle waiting for request
        // This is where you would typically read the request from the client
        // For now, we'll just simulate it with a sleep

        return RequestState::GettingTicket;
    }

    pub async fn handle_getting_ticket(&mut self) -> RequestState {

        let mut desired_mechs = OidSet::new().unwrap();
        desired_mechs.add(&GSS_MECH_KRB5).unwrap();
        
        let parsed_name = match Name::new(&self.kerberos_service, Some(&GSS_NT_HOSTBASED_SERVICE)) {
            Ok(name) => name,
            Err(e) => {
                self.last_error = StateErrors::FailedGettingServiceTicket;
                self.error_message = Some(format!("Failed to prepare TGS request: {:#?}", e));
                return RequestState::Closing;
            }
        };

        let mut client_ctx = match Cred::acquire(None, None, CredUsage::Initiate, Some(&desired_mechs)) {
            Ok(cred) => {
                let ctx = ClientCtx::new(
                    Some(cred), parsed_name, CtxFlags::GSS_C_MUTUAL_FLAG, Some(&GSS_MECH_KRB5)
                );
                ctx
            },
            Err(e) => {
                self.last_error = StateErrors::FailedGettingServiceTicket;
                self.error_message = Some(format!("Failed to create client context: {:#?}", e));
                return RequestState::Closing;
            }
        };

    
        let gss_buffer = match client_ctx.step(None, None) {
            Ok(t) => t,
            Err(e) => {
                self.last_error = StateErrors::FailedGettingServiceTicket;
                self.error_message = Some(format!("Failed to get AP-REQ token: {:#?}", e));
                return RequestState::Closing;
            }
        };
    
    
        let ap_req = match gss_buffer {
            Some(buf) => {
                let b64 = base64::engine::general_purpose::STANDARD.encode(buf.as_ref());
                format!("Negotiate {}", b64)
            },
            None => {
                self.last_error = StateErrors::FailedGettingServiceTicket;
                self.error_message = Some("Failed to encode AP-REQ token".to_string());
                return RequestState::Closing;
            },
        };

        self.ap_req = Some(ap_req.clone());
    
        return RequestState::ConnectingToProxy;
    
    }


    pub async fn handle_connecting_to_proxy(&mut self) -> RequestState {

        if self.original_request.as_ref().unwrap().method() == "CONNECT" {
            println!("Handling CONNECT request");
            return RequestState::Tunelling;
        } else {
            println!("Handling other request");
            return RequestState::Forwarding;
        }
    }

    pub async fn handle_tunelling(&mut self) -> RequestState {


        let mut proxy_stream = match TcpStream::connect((self.proxy_ip.clone(), self.proxy_port)).await  {
            Ok(stream) => stream,
            Err(e) => {
                self.last_error = StateErrors::FailedProxyConnection;
                self.error_message = Some(format!("Failed to connect to proxy: {:#?}", e));
                return RequestState::Closing;
            }
        };

        let proxy_string = self.ap_req.as_ref().unwrap();
        let host = self.original_request.as_ref().unwrap().uri().host().unwrap_or("localhost");

        let connect_req = format!(
            "CONNECT {} HTTP/1.1\r\nHost: {}\r\nProxy-Authorization: {}\r\n\r\n",
            self.original_request.as_ref().unwrap().uri(), host, proxy_string
        );

        match proxy_stream.write_all(connect_req.as_bytes()).await {
            Ok(_) => println!("CONNECT request sent to proxy"),
            Err(e) => {
                self.last_error = StateErrors::FailedProxyConnection;
                self.error_message = Some(format!("Failed to send CONNECT request: {:#?}", e));
                return RequestState::Closing;
            }
        }
        // 3. Přečti odpověď a ověř 200 OK
        let mut buf = [0u8; 1024];
        let n = proxy_stream.read(&mut buf).await.unwrap();
        let proxy_response = String::from_utf8_lossy(&buf[..n]);

        if !proxy_response.starts_with("HTTP/1.1 200") {
            self.last_error = StateErrors::FailedTunnel;
            return RequestState::Closing;
        }

        // Removed try_clone as TcpStream cannot be cloned; self.proxy_stream is not set.


        //let mut upgraded_client = TokioIo::new(upgraded);

        let mut original_request = self.original_request.take().unwrap();
        let on_upgrade = hyper::upgrade::on(&mut original_request);

        let mut resp_to_client = Response::new(empty());
        *resp_to_client.status_mut() = StatusCode::OK;

        self.original_response = Some(resp_to_client.map(|b| b.boxed()));


        tokio::task::spawn(async move {
            let upgraded_client = on_upgrade.await.unwrap();
            let mut client_io = TokioIo::new(upgraded_client);  
            match tokio::io::copy_bidirectional(&mut client_io, &mut proxy_stream).await {
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

        println!("proxy_response: {:?}", proxy_response);

        let mut resp_to_client = Response::new(empty());
        *resp_to_client.status_mut() = StatusCode::OK;

        self.original_response = Some(resp_to_client.map(|b| b.boxed()));

        return RequestState::Closing;

    }



    pub async fn handle_forwarding(&mut self) -> RequestState {


        let ap_req = self.ap_req.as_ref().unwrap();

        let proxy_stream = match TcpStream::connect((self.proxy_ip.clone(), self.proxy_port)).await{
            Ok(stream) => stream,
            Err(e) => {
                self.last_error = StateErrors::FailedProxyConnection;
                self.error_message = Some(format!("Failed to connect to proxy: {:#?}", e));
                return RequestState::Closing;
            }
        };

        let io = TokioIo::new(proxy_stream);

        let (mut proxy_sender, proxy_conn) = match ClientBuilder::new()
            .preserve_header_case(true)
            .title_case_headers(true)
            .handshake(io)
            .await {
                Ok(conn) => conn,
                Err(e) => {
                    self.last_error = StateErrors::FailedProxyConnection;
                    self.error_message = Some(format!("Failed to connect to proxy: {:#?}", e));
                    return RequestState::Closing;
                }
            };

        tokio::task::spawn(async move {
            if let Err(err) = proxy_conn.await {
                println!("Connection failed: {:?}", err);
            }
        });

        let mut original_request = self.original_request.take().unwrap();
        
        original_request.headers_mut().insert(
            hyper::header::PROXY_AUTHORIZATION,
            ap_req.parse().unwrap(),
        );

        let (parts, body) = original_request.into_parts();
        let new_req = http::Request::from_parts(parts, body);
        let resp = proxy_sender.send_request(new_req).await.unwrap();

        self.original_response = Some(resp.map(|b| b.boxed()));

        return RequestState::Closing;
    }

    pub async fn handle_closing(&mut self) -> RequestState {

        // Handle closing - examine states, return errors, clean resources, etc

        return RequestState::Closing;
    }

}

pub fn empty() -> BoxBody<Bytes, hyper::Error> {
    Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed()
}
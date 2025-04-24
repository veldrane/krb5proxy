use libgssapi::oid::{OidSet, GSS_NT_HOSTBASED_SERVICE, GSS_MECH_KRB5};
use krb5proxy::krb5::setup_client_ctx;
use krb5proxy::upstream::connect;
use libgssapi::name::Name;
use base64::engine::Engine;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

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
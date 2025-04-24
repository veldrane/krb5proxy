
use libgssapi::context::{ClientCtx, CtxFlags};
use libgssapi::name::Name;
use libgssapi::credential::{Cred, CredUsage};
use libgssapi::oid::{OidSet, GSS_MECH_KRB5};
use libgssapi::error::Error;
//use base64::{engine::general_purpose, Engine as _};


pub fn setup_client_ctx(service_name: Name, desired_mechs: &OidSet )-> Result<ClientCtx, Error> {
    
    let client_cred = Cred::acquire(
        None, None, CredUsage::Initiate, Some(&desired_mechs)
    ).expect("faild to acquire client credentials");

    println!("acquired default client credentials: {:#?}", client_cred);


    Ok(ClientCtx::new(
        Some(client_cred), service_name, CtxFlags::GSS_C_MUTUAL_FLAG, Some(&GSS_MECH_KRB5)
    ))
}

pub fn generate_kerberos_auth_header(spn: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Sestavíme GSSAPI jméno z SPN (např. HTTP/proxy.local)
    let service_name = Name::new(spn.as_bytes(), None)?;

    // Získáme credentials z aktuální Kerberos cache (např. krb5cc)
    let cred = Cred::acquire(None, None, CredUsage::Initiate, None)?;

    // Vytvoříme GSSAPI kontext pro autentizaci
    let mut ctx = ClientCtx::new(
        Some(cred),
        service_name,
        CtxFlags::GSS_C_MUTUAL_FLAG | CtxFlags::GSS_C_REPLAY_FLAG,
        None,
    );


    // Spustíme handshake – první krok bez předchozího tokenu
    //let token = ctx.step(&Buf::empty())?
    //    .ok_or("No token returned during GSSAPI negotiation")?;

    // Získaný AP-REQ token zakódujeme do base64
    //let base64_token = general_purpose::STANDARD.encode(token.as_slice());

    // Vytvoříme finální hlavičku
    //Ok(format!("Negotiate {}", base64_token))
    Ok(format!("Negotiate"))
}


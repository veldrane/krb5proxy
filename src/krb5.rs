
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

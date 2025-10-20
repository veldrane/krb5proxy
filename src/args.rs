use clap::Parser;


/// A tiny proxy to forward requests with Kerberos authentication to parrent proxy
#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
pub struct Args {
    /// Parrent proxy string in the format <ip>:<port> 
    #[clap(short, long)]
    proxy: String,
    /// Parent proxy hostname
    #[clap(long)]
    proxy_hostname: String,
    /// Listen address in the format http://<ip>:<port> - default is http://127.0.0.1:8080
    #[clap(short, long, default_value = "http://127.0.0.1:8080")]
    listen: String,
    #[clap(short, long, default_value = "console")]
    log: String,

}


impl Args {
    /// Create a new Args instance
    pub fn new() -> Self {
        Args::parse()
    }

    pub fn get_proxy_ip(&self) -> String {
        self.proxy.split(':').next().unwrap_or("").to_string()
    }

    pub fn get_proxy_port(&self) -> u16 {
        self.proxy.split(':').nth(1).unwrap_or("8080").parse().unwrap_or(8080)
    }
    pub fn get_proxy_hostname(&self) -> String {
        self.proxy_hostname.clone()
    }

    pub fn get_kerberos_service(&self) -> String {
        format!("HTTP@{}", self.proxy_hostname)
    }

    pub fn get_listen(&self) -> String {
        self.listen.clone()
    }

    pub fn get_log(&self) -> String {
        self.log.clone()
    }
}
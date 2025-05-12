use std::u16;


#[derive(Debug, Clone)]
pub struct Config {
    pub proxy_ip: String,
    pub proxy_port: u16,
    pub kerberos_service: Vec<u8>,
    pub listen_address: String,
}

impl Config {
    pub fn builder() -> Self {
        Config {
            proxy_ip: "127.0.0.1".to_string(),
            proxy_port: 8080,
            kerberos_service: b"HTTP@localhost".to_vec(),
            listen_address: "http://127.0.0.1:8080".to_string(),
        }
    }

    pub fn with_proxy_ip(mut self, ip: String) -> Self {
        self.proxy_ip = ip;
        self
    }

    pub fn with_proxy_port(mut self, port: u16) -> Self {
        self.proxy_port = port;
        self
    }

    pub fn with_kerberos_service(mut self, service: String) -> Self {
        self.kerberos_service = service.as_bytes().to_vec();
        self
    }

    pub fn with_listen_address(mut self, address: String) -> Self {
        self.listen_address = address;
        self
    }

    pub fn build(self) -> Self {
        self
    }

    pub fn get_listen_ip(&self) -> String {

        let listen_ip = match self.listen_address.split("://").nth(1) {
            Some(addr) => {
                addr.split(':').nth(0).unwrap_or("127.0.0.1")},
            None => "127.0.0.1",
        };

        listen_ip.to_string()
    }

    pub fn get_listen_port(&self) -> u16 {
        match self.listen_address.split(":").nth(2) {
            Some(port) => port.parse::<u16>().unwrap_or(8080),
            None => 8080,
        }
    }
}
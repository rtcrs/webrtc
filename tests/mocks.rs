
#[allow(dead_code)]
pub mod dtls {

    use super::transport;

    use tokio::time::Duration;

    pub type CipherSuite = u16;
    pub const TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256: CipherSuite = 0;
    pub const TLS_ECDHE_ECDSA_WITH_AES_256_CBC_SHA: CipherSuite    = 0;
    pub const TLS_PSK_WITH_AES_128_CCM: CipherSuite                = 0;
    pub const TLS_PSK_WITH_AES_128_CCM_8: CipherSuite              = 0;
    pub const TLS_PSK_WITH_AES_128_GCM_SHA256: CipherSuite         = 0;
    pub const TLS_ECDHE_ECDSA_WITH_AES_128_CCM: CipherSuite        = 0;
    pub const TLS_ECDHE_ECDSA_WITH_AES_128_CCM_8: CipherSuite      = 0;
    pub const TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256: CipherSuite   = 0;
    pub const TLS_ECDHE_RSA_WITH_AES_256_CBC_SHA: CipherSuite      = 0;

    // TODO
    pub const REQUIRE_ANY_CLIENT_CERT: () = ();

    type FlightInterval = Duration;
    pub type MTU = u16;
    // TODO
    pub type PSK = ();
    pub type PSKIdHint = ();

    const BACKOFF: Duration = Duration::from_millis(500);

    pub struct Client {
        conn: transport::Connection,
        config: Config,
    }

    impl Client {
        pub fn new(conn: transport::Connection, config: Config) -> Result<Self, String> {
            Ok( Client {
                conn,
                config,
            })
        }
        pub async fn start(&self) {
            println!("client started")
        }
        pub async fn next(&self) -> Event {
            println!("client polled");
            Event::Message { content: () }
        }
        pub fn get_connection(&self) -> &transport::Connection {
            &self.conn
        }
    }

    pub struct Server {
        conn: transport::Connection,
        config: Config,
    }

    impl Server {
        pub fn new(conn: transport::Connection, config: Config) -> Result<Self, String> {
            Ok( Server {
                conn,
                config,
            })
        }
        pub async fn start(&self) {
            println!("server started")
        }
        pub async fn next(&self) -> Event {
            println!("server polled");
            Event::Message { content: () }
        }
        pub fn get_connection(&self) -> &transport::Connection {
            &self.conn
        }
    }

    #[derive(Debug)]
    pub enum Event {
        Message { content: () },
        Error { reason: () },
    }

    #[derive(Debug)]
    #[derive(Clone)]
    #[derive(Copy)]
    pub struct Cert {
        config: CertConfig
    }

    impl Cert {
        pub fn new(config: CertConfig) -> Self { Cert { config } }
    }

    #[derive(Debug)]
    #[derive(Clone)]
    #[derive(Copy)]
    pub struct CertConfig {
        host: Option<&'static str>,
        self_signed: bool,
    }

    impl CertConfig {
        pub fn new() -> Self {
            CertConfig {
                host: None,
                self_signed: false,
            }
        }
        pub fn host(&self, host: &'static str) -> Self {
            CertConfig {
                host: Some(host),
                self_signed: self.self_signed,
            }
        }
        pub fn self_signed(&self) -> Self {
            CertConfig {
                host: self.host,
                self_signed: true,
            }
        }
    }

    #[derive(Debug)]
    #[derive(Clone)]
    #[derive(Copy)]
    pub enum ClientAuthType {
        NoClientCert,
        RequireAnyClientCert,
    }

    #[derive(Debug)]
    #[derive(Clone)]
    #[derive(Copy)]
    pub struct Config {
        pub cipher_suite: Option<CipherSuite>,
        pub cert: Option<Cert>,
        pub insecure_skip_verify: bool,
        pub psk: Option<PSK>,
        pub psk_id_hint: Option<PSKIdHint>,
        pub mtu: Option<MTU>,
        pub flight_interval: Option<FlightInterval>,
        pub client_auth_type: ClientAuthType,
    }

    // TODO: there is almost definitely an existing macro for this...
    impl Config {
        pub fn new() -> Self {
            Config {
                cipher_suite: None,
                cert: None,
                insecure_skip_verify: false,
                psk: None,
                psk_id_hint: None,
                mtu: None,
                flight_interval: None,
                client_auth_type: ClientAuthType::NoClientCert,
            }
        }

        pub fn cert(&self, cert: Cert) -> Self {
             Config {
                cipher_suite: self.cipher_suite,
                cert: Some(cert),
                insecure_skip_verify: self.insecure_skip_verify,
                psk: self.psk,
                psk_id_hint: self.psk_id_hint,
                mtu: self.mtu,
                flight_interval: None,
                client_auth_type: self.client_auth_type,
            }
        }

        pub fn cipher_suite(&self, cipher_suite: CipherSuite) -> Self {
            Config {
                cipher_suite: Some(cipher_suite),
                cert: self.cert,
                insecure_skip_verify: self.insecure_skip_verify,
                psk: self.psk,
                psk_id_hint: self.psk_id_hint,
                mtu: self.mtu,
                flight_interval: None,
                client_auth_type: self.client_auth_type,
            }
        }

        pub fn insecure_skip_verify(&self) -> Self {
            Config {
                cipher_suite: self.cipher_suite,
                cert: self.cert,
                insecure_skip_verify: true,
                psk: self.psk,
                psk_id_hint: self.psk_id_hint,
                mtu: self.mtu,
                flight_interval: None,
                client_auth_type: self.client_auth_type,
            }
        }

        pub fn psk(&self, psk: PSK, psk_id_hint: PSKIdHint) -> Self {
            Config {
                cipher_suite: self.cipher_suite,
                cert: self.cert,
                insecure_skip_verify: self.insecure_skip_verify,
                psk: Some(psk),
                psk_id_hint: Some(psk_id_hint),
                mtu: self.mtu,
                flight_interval: None,
                client_auth_type: self.client_auth_type,
            }
        }

        pub fn mtu(&self, mtu: MTU) -> Self {
            Config {
                cipher_suite: self.cipher_suite,
                cert: self.cert,
                insecure_skip_verify: self.insecure_skip_verify,
                psk: self.psk,
                psk_id_hint: self.psk_id_hint,
                mtu: Some(mtu),
                flight_interval: None,
                client_auth_type: self.client_auth_type,
            }
        }

        pub fn flight_interval(&self, flight_interval: FlightInterval) -> Self {
            Config {
                cipher_suite: self.cipher_suite,
                cert: self.cert,
                insecure_skip_verify: self.insecure_skip_verify,
                psk: self.psk,
                psk_id_hint: self.psk_id_hint,
                mtu: self.mtu,
                flight_interval: Some(flight_interval),
                client_auth_type: self.client_auth_type,
            }
        }

        pub fn client_auth_type(&self, client_auth_type: ClientAuthType) -> Self {
            Config {
                cipher_suite: self.cipher_suite,
                cert: self.cert,
                insecure_skip_verify: self.insecure_skip_verify,
                psk: self.psk,
                psk_id_hint: self.psk_id_hint,
                mtu: self.mtu,
                flight_interval: None,
                client_auth_type: client_auth_type,
            }
        }
    }

    pub async fn listen(
        _proto: String,
        addr: String,
        port: u16,
        _config: Config,
    ) -> Result<tokio::net::TcpListener, std::io::Error> {
        println!("mock dtls::listen on {}:{}", addr, port);
        tokio::net::TcpListener::bind(format!("{}:{}", addr, port)).await
    }

    pub async fn dial(
        _proto: String,
        addr: String,
        port: u16,
        _config: Config,
    ) -> Result<tokio::net::TcpStream, std::io::Error> {
        println!("mock dtls::dial on {}:{}", addr, port);
        tokio::net::TcpStream::connect(format!("{}:{}", addr, port)).await
    }

}

#[allow(dead_code)]
#[allow(unused_variables)]
pub mod transport {

    #[derive(Copy)]
    #[derive(Clone)]
    pub struct Connection { }

    impl Connection {
        pub fn new() -> Self { Connection { } }
        pub fn send(&self, message: &str) -> Result<u16, &str> { Ok(0) }
        pub fn recv(&self, buffer: &mut [u8; 8192]) -> Result<usize, &str> { Ok(0) }
    }

    #[derive(Copy)]
    #[derive(Clone)]
    pub struct Bridge { }
    
    impl Bridge {
        pub fn new() -> Self { Bridge { } }
        pub fn set_loss_chance(&self, loss_chance: u8) { }
        pub fn get_connection(&self) -> Connection { Connection { } }
    }
}

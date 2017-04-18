use serde_yaml;

use std::path::Path;
use std::io::prelude::*;
use std::fs::File;

use std::result::Result;

use opcua_core::types::MessageSecurityMode;
use constants;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct TcpConfig {
    /// Timeout for hello on a session in seconds
    pub hello_timeout: u32,
    /// The hostname to supply in the endpoints
    pub host: String,
    /// The port number of the service
    pub port: u16,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ServerEndpoint {
    /// Name for the endpoint
    pub name: String,
    /// Endpoint path
    pub path: String,
    /// Security policy
    pub security_policy: String,
    /// Security mode
    pub security_mode: String,
    /// Allow anonymous access (default false)
    pub anonymous: Option<bool>,
    /// Allow user name / password access
    pub user: Option<String>,
    pub pass: Option<String>,
}

const DEFAULT_ENDPOINT_NAME: &'static str = "Default";
const DEFAULT_ENDPOINT_PATH: &'static str = "/";

const DEFAULT_SECURITY_POLICY: &'static str = SECURITY_POLICY_NONE;
const SECURITY_POLICY_NONE: &'static str = "None";
const SECURITY_POLICY_BASIC_128_RSA_15: &'static str = "Basic128Rsa15";
const SECURITY_POLICY_BASIC_256: &'static str = "Basic256";
const SECURITY_POLICY_BASIC_256_SHA_256: &'static str = "Basic256Sha256";

const DEFAULT_SECURITY_MODE: &'static str = SECURITY_MODE_NONE;
const SECURITY_MODE_NONE: &'static str = "None";
const SECURITY_MODE_SIGN: &'static str = "Sign";
const SECURITY_MODE_SIGN_AND_ENCRYPT: &'static str = "SignAndEncrypt";

impl ServerEndpoint {
    pub fn new(name: &str, path: &str, anonymous: bool, user: &str, pass: &[u8], security_policy: &str, security_mode: &str) -> ServerEndpoint {
        ServerEndpoint {
            name: name.to_string(),
            path: path.to_string(),
            anonymous: Some(anonymous),
            user: if user.is_empty() { None } else { Some(user.to_string()) },
            pass: if user.is_empty() { None } else { Some(String::from_utf8(pass.to_vec()).unwrap()) },
            security_policy: security_policy.to_string(),
            security_mode: security_mode.to_string(),
        }
    }

    pub fn new_default(anonymous: bool, user: &str, pass: &[u8], security_policy: &str, security_mode: &str) -> ServerEndpoint {
        ServerEndpoint::new(DEFAULT_ENDPOINT_NAME, DEFAULT_ENDPOINT_PATH, anonymous, user, pass, security_policy, security_mode)
    }

    pub fn default_anonymous() -> ServerEndpoint {
        ServerEndpoint::new_default(true, "", &[], DEFAULT_SECURITY_POLICY, DEFAULT_SECURITY_MODE)
    }

    pub fn default_user_pass(user: &str, pass: &[u8]) -> ServerEndpoint {
        ServerEndpoint::new_default(false, user, pass, DEFAULT_SECURITY_POLICY, DEFAULT_SECURITY_MODE)
    }

    /// Special config that turns on anonymous, user/pass and pki for sample code that wants everything available
    /// Don't use in production.
    pub fn default_sample() -> ServerEndpoint {
        ServerEndpoint::new_default(true, "sample", "sample1".as_bytes(), DEFAULT_SECURITY_POLICY, DEFAULT_SECURITY_MODE)
    }

    pub fn is_valid(&self) -> bool {
        let mut valid = true;
        if (self.user.is_some() && self.pass.is_none()) || (self.user.is_none() && self.pass.is_some()) {
            error!("Endpoint {} is invalid. User / password both need to be set or not set, not just one or the other", self.name);
            valid = false;
        }

        match self.security_policy.as_ref() {
            SECURITY_POLICY_NONE | SECURITY_POLICY_BASIC_128_RSA_15 | SECURITY_POLICY_BASIC_256 | SECURITY_POLICY_BASIC_256_SHA_256 => {}
            _ => {
                error!("Endpoint {} is invalid. Security policy \"{}\" is invalid. Valid values are None, Basic128Rsa15, Basic256, Basic256Sha256", self.name, self.security_policy);
                valid = false;
            }
        }

        match self.security_mode.as_ref() {
            SECURITY_MODE_NONE | SECURITY_MODE_SIGN | SECURITY_MODE_SIGN_AND_ENCRYPT => {}
            _ => {
                error!("Endpoint {} is invalid. Security mode \"{}\" is invalid. Valid values are None, Sign, SignAndEncrypt", self.name, self.security_mode);
                valid = false;
            }
        }

        if (&self.security_policy == SECURITY_POLICY_NONE && &self.security_mode != SECURITY_MODE_NONE) ||
            (&self.security_policy != SECURITY_POLICY_NONE && &self.security_mode == SECURITY_MODE_NONE) {
            error!("Endpoint {} is invalid. Security policy and security mode must both contain None or neither of them should.", self.name);
            valid = false;
        }

        if &self.security_policy == SECURITY_POLICY_NONE && &self.security_mode == SECURITY_MODE_NONE && (self.anonymous.is_none() || !self.anonymous.as_ref().unwrap()) {
            error!("Endpoint {} is invalid. Security policy and mode allow anonymous connections but anonymous is not set to true", self.name);
            valid = false;
        }

        valid
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ServerConfig {
    /// An id for this server
    pub application_name: String,
    /// A description for this server
    pub application_uri: String,
    /// Product url
    pub product_uri: String,
    /// pki folder, either absolute or relative to executable
    pub pki_dir: String,
    /// Flag turns on or off discovery service
    pub discovery_service: bool,
    /// tcp configuration information
    pub tcp_config: TcpConfig,
    /// Endpoints supported by the server
    pub endpoints: Vec<ServerEndpoint>,
    /// Max array length in elements
    pub max_array_length: u32,
    /// Max string length in characters
    pub max_string_length: u32,
    /// Max bytestring length in bytes
    pub max_byte_string_length: u32,
}

impl ServerConfig {
    pub fn default(endpoints: Vec<ServerEndpoint>) -> ServerConfig {
        let application_name = "OPCUA-Rust".to_string();
        let hostname = "127.0.0.1".to_string();

        let application_uri = format!("urn:{}", application_name);
        let product_uri = format!("urn:{}", application_name);

        ServerConfig {
            application_name: application_name,
            application_uri: application_uri,
            product_uri: product_uri,
            discovery_service: true,
            pki_dir: "pki".to_string(),
            tcp_config: TcpConfig {
                host: hostname,
                port: constants::DEFAULT_OPC_UA_SERVER_PORT,
                hello_timeout: constants::DEFAULT_HELLO_TIMEOUT_SECONDS,
            },
            endpoints: endpoints,
            max_array_length: constants::DEFAULT_MAX_ARRAY_LENGTH,
            max_string_length: constants::DEFAULT_MAX_STRING_LENGTH,
            max_byte_string_length: constants::DEFAULT_MAX_BYTE_STRING_LENGTH,
        }
    }

    /// Returns the default server configuration to run a server with no security and anonymous access enabled
    pub fn default_anonymous() -> ServerConfig {
        ServerConfig::default(vec![ServerEndpoint::default_anonymous()])
    }

    pub fn default_user_pass(user: &str, pass: &[u8]) -> ServerConfig {
        ServerConfig::default(vec![ServerEndpoint::default_user_pass(user, pass)])
    }

    pub fn default_sample() -> ServerConfig {
        ServerConfig::default(vec![ServerEndpoint::default_sample()])
    }

    pub fn save(&self, path: &Path) -> Result<(), ()> {
        if self.is_valid() {
            let s = serde_yaml::to_string(&self).unwrap();
            if let Ok(mut f) = File::create(path) {
                if f.write_all(s.as_bytes()).is_ok() {
                    return Ok(());
                }
            }
        }
        Err(())
    }

    pub fn load(path: &Path) -> Result<ServerConfig, ()> {
        if let Ok(mut f) = File::open(path) {
            let mut s = String::new();
            if f.read_to_string(&mut s).is_ok() {
                if let Ok(config) = serde_yaml::from_str(&s) {
                    return Ok(config)
                }
            }
        }
        Err(())
    }

    pub fn is_valid(&self) -> bool {
        let mut valid = true;
        if self.endpoints.is_empty() {
            error!("Server configuration is invalid. It defines no endpoints");
            valid = false;
        }
        for e in self.endpoints.iter() {
            if !e.is_valid() {
                valid = false;
            }
        }
        if self.max_array_length == 0 {
            error!("Server configuration is invalid.  Max array length is invalid");
            valid = false;
        }
        if self.max_string_length == 0 {
            error!("Server configuration is invalid.  Max string length is invalid");
            valid = false;
        }
        if self.max_byte_string_length == 0 {
            error!("Server configuration is invalid.  Max byte string length is invalid");
            valid = false;
        }
        valid
    }

    pub fn message_security_mode() -> MessageSecurityMode {
        MessageSecurityMode::None
    }
}
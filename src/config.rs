/// Module to obtain the server settings.
///
/// Those will be read from the toml configuration file, for the
/// ones that are not found a default will be used.
///
/// ```toml
/// previews_path = "/var/doc-previewer"
/// retention_days = 14
/// max_artifact_size = 524288000
///
/// [server]
/// address = "0.0.0.0"
/// port = 8000
/// url = "https://doc-previewer.pydata.org/"
///
/// [github]
/// entrypoint = "https://api.github.com/repos/"
/// token = "xxxxx"
/// allowed_owners = [ "pydata", "pandas-dev" ]
///
/// [log]
/// level = "info"
/// ```
use std::fs;
use std::path::Path;
use std::collections::HashSet;
use serde_derive::Deserialize;

const PREVIEWS_PATH: &str = "/var/doc-previewer";
const RETENTION_DAYS: f64 = 14.;
const MAX_ARTIFACT_SIZE: usize = 500 * 1024 * 1024; // 500 Mb

const SERVER_ADDRESS: &str = "0.0.0.0";
const SERVER_PORT: u16 = 8000;
const SERVER_URL: &str = "https://doc-previewer.pydata.org/";

const GITHUB_ENDPOINT: &str = "https://api.github.com/repos/";

const LOG_LEVEL: &str = "info";

#[derive(Deserialize)]
pub struct TomlConfig {
    previews_path: Option<String>,
    retention_days: Option<f64>,
    max_artifact_size: Option<usize>,
    server: TomlServer,
    github: TomlGitHub,
    log: Option<TomlLog>
}

#[derive(Deserialize)]
struct TomlServer {
    address: Option<String>,
    port: Option<u16>,
    url: Option<String>
}

#[derive(Deserialize)]
struct TomlGitHub {
    endpoint: Option<String>,
    token: String,
    allowed_owners: Vec<String>
}

#[derive(Deserialize)]
struct TomlLog {
    level: Option<String>
}

/// Settings after filling the missing ones with default values.
pub struct Settings {
    pub server_address: String,
    pub server_port: u16,

    pub github_token: String,

    pub log_level: String,

    pub per_thread: SettingsPerThread
}

/// Settings that will be cloned for each thread
#[derive(Clone)]
pub struct SettingsPerThread {
    pub previews_path: String,
    pub retention_days: f64,
    pub max_artifact_size: usize,

    pub server_url: String,

    pub github_endpoint: String,
    pub github_allowed_owners: HashSet<String>
}

impl Settings {
    pub fn load(fname: &Path) -> Self {
        let config_content = fs::read_to_string(fname).unwrap_or_else(
            |_| panic!("Configuration file {:?} not found", fname)
        );
        let config: TomlConfig = toml::from_str(&config_content).unwrap();
        let settings = Settings {
            server_address: config.server.address.unwrap_or(SERVER_ADDRESS.to_owned()),
            server_port: config.server.port.unwrap_or(SERVER_PORT),

            github_token: config.github.token.to_owned(),

            log_level: config.log.as_ref().and_then(|x| x.level.to_owned()).unwrap_or(LOG_LEVEL.to_owned()),

            per_thread: SettingsPerThread {
                previews_path: config.previews_path.unwrap_or(PREVIEWS_PATH.to_owned()),
                retention_days: config.retention_days.unwrap_or(RETENTION_DAYS),
                max_artifact_size: config.max_artifact_size.unwrap_or(MAX_ARTIFACT_SIZE),

                server_url: config.server.url.unwrap_or(SERVER_URL.to_owned()),

                github_endpoint: config.github.endpoint.unwrap_or(GITHUB_ENDPOINT.to_owned()),
                github_allowed_owners: config.github.allowed_owners.into_iter().collect()
            }
        };
        settings
    }
}

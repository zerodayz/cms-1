use toml;
use serde::Deserialize;
use lazy_static::lazy_static;

lazy_static! {
    // The config variable is declared as an immutable static.
    pub static ref CONFIG: Config = Config::new();
}

#[derive(Deserialize)]
pub struct Config {
    pub default: Default,
    pub database: Database,
}

#[derive(Deserialize)]
pub struct Default {
    pub title: String,
    pub description: String,
}

#[derive(Deserialize)]
pub struct Database {
    pub name: String,
    pub password: String,
    pub host: String,
    pub port: u16,
    pub user: String,
}

impl Config {
    pub fn new() -> Self {
        // Initialize the config's fields.
        let config_path = "config.toml";
        let config = std::fs::read_to_string(config_path).expect("Unable to read config.toml");
        let config: Config = toml::from_str(&config).unwrap();

        Self {
            default: Default {
                title: config.default.title,
                description: config.default.description
            },
            database: Database {
                name: config.database.name,
                password: config.database.password,
                host: config.database.host,
                port: config.database.port,
                user: config.database.user
            }
        }
    }
}

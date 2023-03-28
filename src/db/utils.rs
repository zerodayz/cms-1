use crate::config::config::Config;

use thiserror::Error;
use diesel::prelude::*;
use diesel::mysql::MysqlConnection;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Database connection error.")]
    DatabaseConnectionError,
}

pub fn establish_connection(config: &Config) -> Result<MysqlConnection, Error> {
    let database_url = format!("mysql://{}:{}@{}:{}/{}",
                               &config.database.user,
                               &config.database.password,
                               &config.database.host,
                               &config.database.port,
                               &config.database.name);

    MysqlConnection::establish(&database_url)
        .map_err(|_| Error::DatabaseConnectionError)
}
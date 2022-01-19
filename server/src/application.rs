//! Application level setup and configuration

use std::fs::{File, OpenOptions};
use std::io::BufReader;
use std::path::PathBuf;
use std::{fmt::Debug, path::Path};

use serde::{Deserialize, Serialize};
use tracing_subscriber::{
    filter::{LevelFilter, Targets},
    prelude::*,
    util::SubscriberInitExt,
    Layer,
};

use crate::prelude::*;

pub const DEFAULT_CONFIG_LOCATION: &str = "./server.config.yml";

#[derive(clap::Parser, Clone, Debug)]
pub enum SubCommand {
    /// Validate configuration files
    Validate {
        /// Create defaults for required configuration if files do not exist
        #[clap(short, long)]
        creating: bool,
    },
}

#[derive(clap::Parser, Clone)]
pub struct Args {
    /// Path to configuration file
    #[clap(long = "config")]
    pub config_location: Option<PathBuf>,

    #[clap(subcommand)]
    pub subcommand: Option<SubCommand>,
}

impl Args {
    pub fn config_location_or_default(&self) -> PathBuf {
        if let Some(ref config_location) = self.config_location {
            config_location.clone()
        } else {
            PathBuf::from(DEFAULT_CONFIG_LOCATION)
        }
    }
}

impl Debug for Args {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // This is done manually so that we can show computed values as well

        f.debug_struct("Args")
            .field("config_location", &self.config_location)
            .field("computed_config", &self.config_location_or_default())
            .field("subcommand", &self.subcommand)
            .finish()
    }
}

pub fn init_tracing() {
    let log_layer = tracing_subscriber::fmt::layer().with_filter(
        // TODO(josiah) monitor upstream for or contribute a from_env/from_env_default for
        // `Targets`. Conversations in the discord indicate that what I have here would be a
        // reasonable default.
        std::env::var("RUST_LOG")
            .map(|it| it.parse().expect("Could not parse value of RUST_LOG"))
            .unwrap_or_else(|_| Targets::default().with_default(LevelFilter::INFO)),
    );

    let console_layer = console_subscriber::spawn();

    // TODO(josiah) Consider adding in log file which always logs with TRACE

    tracing_subscriber::registry()
        .with(log_layer)
        .with(console_layer)
        .init();
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Configuration {
    pub users: PathBuf,
}

impl Configuration {
    /// Attempt to read the configuration from file
    pub fn get(path: impl AsRef<Path>) -> Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        serde_yaml::from_reader(reader).map_err(Into::into)
    }

    /// Writ this configuration to file
    pub fn put(&self, path: impl AsRef<Path>) -> Result<()> {
        let file = OpenOptions::new().create(true).write(true).open(path)?;
        serde_yaml::to_writer(file, self).map_err(Into::into)
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            users: PathBuf::from("./users.yml"),
        }
    }
}

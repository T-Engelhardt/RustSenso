use std::fmt;

use clap::Parser;
use const_format::formatcp;
use env_logger::Env;
use log::{error, info};
use senso::{
    connector::Connector,
    urls::UrlBase,
};

// THIS PART IS THE SAME AS SENSOR
// ON CHANGES DONT FORGET TO CHANGE SENSOR TOO
// START
pub const VERSION_STR: &str =
    formatcp!("v{}, senso v{}", env!("CARGO_PKG_VERSION"), senso::VERSION);

/// Insert vaillant api usage data from a facility into a sqlite database.
#[derive(Parser)]
#[command(version = VERSION_STR, about, long_about = None)]
struct Args {
    /// Specify the serial of the facility.
    #[arg(short, long)]
    serial: String,

    /// Path of the Sqlite file.
    /// Creates a new file if not found.
    #[arg(short, long, default_value = "./data.db")]
    db_file: String,

    /// User name for login.
    #[arg(long)]
    user: String,

    /// Password for login.
    #[arg(long)]
    pwd: String,

    /// Path to token file.
    /// Creates a new file if not found.
    #[arg(short, long, default_value = "./token")]
    token_file: String,
}

impl fmt::Display for Args {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "serial: {}\ndb_path: {}\ntoken_path: {}\nuser: {}\npwd: ###",
            self.serial, self.db_file, self.token_file, self.user
        )
    }
}

fn main() {
    let args = Args::parse();

    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    info!("{} {}", env!("CARGO_PKG_NAME"), VERSION_STR);
    info!("Starting {} with: \n{}", env!("CARGO_PKG_NAME"), args);

    let mut c = Connector::new(UrlBase::VaillantSensoApi, args.serial, args.token_file);
    if c.login(&args.user, &args.pwd)
        .map_err(|e| error!("{}", e.to_string()))
        .is_err()
    {
        error!("Failed to login.");
        return;
    }
    // END SAME AS

    todo!("");
}
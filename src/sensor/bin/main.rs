use std::fmt;

use clap::Parser;
use env_logger::Env;
use log::{debug, error, info};
use senso::{
    connector::Connector,
    db::{SensorData, DB},
    urls::UrlBase,
};

/// Insert vaillant api sensor data from a facility into a sqlite database.
#[derive(Parser)]
#[command(version, about, long_about = None)]
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

    info!("Starting sensor with: \n{}", args);

    let mut c = Connector::new(UrlBase::VaillantAPI, args.serial, args.token_file);
    if c.login(&args.user, &args.pwd)
        .map_err(|e| error!("{}", e.to_string()))
        .is_err()
    {
        error!("Failed to login.");
        return;
    }

    let status = c.system_status().map_err(|e| error!("{}", e.to_string()));
    debug!("{:#?}", status);

    let live_report = c.live_report().map_err(|e| error!("{}", e.to_string()));
    debug!("{:#?}", live_report);

    let data = SensorData::new(&status, &live_report);

    info!("Got Sensor Data: {:#?}", &data);

    if let Ok(db) = DB::new(Some(&args.db_file)).map_err(|e| error!("{}", e.to_string())) {
        if db
            .insert_sensor_data(data)
            .map_err(|e| error!("{}", e.to_string()))
            .is_err()
        {
            error!("Could no insert sensor data in database.")
        }
    } else {
        error!("Failed to open database.")
    }
}

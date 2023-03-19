use std::fmt;

use chrono::{Datelike, Duration};
use clap::Parser;
use cli_table::{print_stdout, WithTitle};
use const_format::formatcp;
use env_logger::Env;
use log::{error, info};
use senso::{
    connector::Connector,
    response::emf_devices::{EmfFunction, EmfType},
    urls::UrlBase,
    yp::{self, UsageFunctionWeek},
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

    /// print cli_table in stdout
    #[arg(short, long, default_value_t = true)]
    cli_table: bool,

    /// how many days back from today in UTC.
    /// 1 => yesterday
    #[arg(long, default_value_t = 1)]
    delta: i64,
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
    // devices from emf_devices
    let devices = vec![
        (
            EmfType::HeatPump,
            "NoneGateway-LL_HMU03_0351_HP_Platform_Outdoor_Monobloc_PR_EBUS",
        ),
        (
            EmfType::Boiler,
            "NoneGateway-LL_VWZ02_0351_HP_Platform_Indoor_Monobloc_PR_EBUS",
        ),
    ];

    let yesterday = chrono::offset::Utc::now() - Duration::days(args.delta);

    let week_nr = yesterday.iso_week().week();
    let year = yesterday.year();

    let mut usage_ch = UsageFunctionWeek::new(EmfFunction::CentralHeating, &devices, year, week_nr);
    let mut usage_dhw =
        UsageFunctionWeek::new(EmfFunction::DomesticHotWater, &devices, year, week_nr);

    if usage_ch
        .retrieve_data(&c)
        .map_err(|e| error!("{}", e.to_string())).is_err() {
            error!("Failed to retrieve data for central heating");
            return;
        }
    if usage_dhw
        .retrieve_data(&c)
        .map_err(|e| error!("{}", e.to_string())).is_err() {
            error!("Failed to retrieve data for domestic hot water");
            return;
        }

    if let Ok(result) =
        yp::build_yp_data_vec(usage_dhw, usage_ch).map_err(|e| error!("{}", e.to_string()))
    {
        if args.cli_table {
            print_stdout(result.with_title()).unwrap();
        }
    } else {
        error!("Failed to create yp data.");
    }
}

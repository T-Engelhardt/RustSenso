use std::fmt;

use chrono::{Datelike, Duration};
use clap::Parser;
use cli_table::{print_stdout, WithTitle};
use const_format::formatcp;
use env_logger::Env;
use log::{error, info};
use senso::{
    connector::Connector,
    db::DB,
    response::emf_devices::{EmfDevice, EmfFunction},
    urls::UrlBase,
    yp::{self, UsageFunctionWeek},
};

// THIS PART IS THE SAME AS SENSOR
// ON CHANGES DONT FORGET TO CHANGE SENSOR TOO
// START
pub const VERSION_STR: &str =
    formatcp!("v{}, senso v{}", env!("CARGO_PKG_VERSION"), senso::VERSION);

/// Insert vaillant api usage data from a facility into a sqlite database.
/// Prints to stdout if no db_file is set.
#[derive(Parser)]
#[command(version = VERSION_STR, about, long_about = None)]
struct Args {
    /// Specify the serial of the facility.
    #[arg(short, long)]
    serial: String,

    /// Path of the Sqlite file.
    /// Creates a new file if not found.
    #[arg(short, long)]
    db_file: Option<String>,

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

    /// how many days back from today in UTC.
    /// 1 => yesterday
    #[arg(long, default_value_t = 1)]
    delta: i64,
}

impl fmt::Display for Args {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "serial: {}\ndb_path: {:#?}\ntoken_path: {}\nuser: {}\npwd: ###",
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
            EmfDevice::HeatPump,
            "NoneGateway-LL_HMU03_0351_HP_Platform_Outdoor_Monobloc_PR_EBUS",
        ),
        (
            EmfDevice::Boiler,
            "NoneGateway-LL_VWZ02_0351_HP_Platform_Indoor_Monobloc_PR_EBUS",
        ),
    ];

    let yesterday = chrono::offset::Utc::now() - Duration::days(args.delta);

    let week_nr = yesterday.iso_week().week();
    let year = yesterday.year();
    let day = yesterday.weekday().num_days_from_monday();

    let mut usage_ch = UsageFunctionWeek::new(EmfFunction::CentralHeating, &devices, year, week_nr);
    let mut usage_dhw =
        UsageFunctionWeek::new(EmfFunction::DomesticHotWater, &devices, year, week_nr);

    if usage_ch
        .retrieve_data(&c)
        .map_err(|e| error!("{}", e.to_string()))
        .is_err()
    {
        error!("Failed to retrieve data for central heating");
        return;
    }
    if usage_dhw
        .retrieve_data(&c)
        .map_err(|e| error!("{}", e.to_string()))
        .is_err()
    {
        error!("Failed to retrieve data for domestic hot water");
        return;
    }

    if let Ok(result) =
        yp::build_yp_data_vec(usage_dhw, usage_ch).map_err(|e| error!("{}", e.to_string()))
    {
        if let Some(db_file) = &args.db_file {
            if let Ok(db) = DB::new(Some(db_file)).map_err(|e| error!("{}", e.to_string())) {
                if db
                    .insert_yp_data(&result[day as usize])
                    .map_err(|e| error!("{}", e.to_string()))
                    .is_err()
                {
                    error!("Could no insert yp data in database.")
                }
            } else {
                error!("Failed to open database.")
            }
        } else {
            // no db file was given, print to stdout
            let _ = print_stdout(result.with_title());
        }
    } else {
        error!("Failed to create yp data.");
    }
}

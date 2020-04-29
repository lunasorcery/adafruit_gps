extern crate adafruit_gps;

use std::env;

pub use adafruit_gps::gps::{GetGpsData, Gps, open_port};
use adafruit_gps::PMTK::send_pmtk::SendPmtk;

fn main() {
    let args = env::args();

    let port = open_port("/dev/serial0");
    let mut gps = Gps { port };

    let update_r = gps.pmtk_220_set_nmea_updaterate(args.get(1));
    dbg!(update_r);

    for _ in 0..10 {
        let values = gps.update();
        // let _pretty_print = format!("utc:{},lat:{:?},long:{:?}, alt:{:?}, course true:{:?}, course mag:{:?}, knots:{:?}, kph:{:?}, geo:{:?}, age:{:?}, sats:{:?}, hdop:{:?}, vdop:{:?}, pdop:{:?}, satellites:{:?}\n", values.utc, values.latitude, values.longitude, values.altitude, values.true_course,
        // values.mag_course, values.speed_knots, values.speed_kph, values.geoidal_spe, values.age_diff_corr,
        // values.sats_used, values.hdop, values.vdop, values.pdop, values.satellites);
        println!("{}", values.utc);
    }
}
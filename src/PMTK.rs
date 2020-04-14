//! pmtk commands are ways of setting the gps.
//!
//! All commands will be destructive for all sentences in the buffer.
//!
//! The PMTK001 command is the response given when there is a valid command given.
//!     It's format is $PMTK001,Command it was given, Flag response (0-3), value passed to it*checksum
//!
//! 5** pmtk standards are all reply formats.
//!
//! If the command given is not valid or supported, failed response is given.
//!
//! Upon any kind of restart the following is printed:
//! $CDACK,7,0*49\r\n  -> Unknown what this is.
//! This is what is given upon wake up.
//! $PMTK011,MTKGPS*08\r\n -> Output sys message.
//! $PMTK010,001*2E\r\n -> Sys message, 001 Startup.
//! $PMTK011,MTKGPS*08\r\n -> 001 txt message, output system message.
//! $PMTK010,002*2D\r\n -> Sys msg, 002 = Notification, aiding EPO.

pub mod send_pmtk {
    use std::str;

    use crate::gps::{GetGpsData, Gps, is_valid_checksum};

    pub enum Pmtk001Ack {
        // format: $PMTK001,cmd,flag*checksum\r\n
        //flag: 0
        Invalid,

        //flag: 1
        Unsupported,

        //flag: 2
        Failed,

        //flag: 3
        Success,

        NoPacket,
    }

    pub enum DgpsMode {
        NoDgps,
        RTCM,
        WAAS,
        Unknown,
    }

    pub enum Sbas {
        Enabled,
        Disabled,
        Unknown,
    }

    pub enum SbasMode {
        Testing,
        Integrity,
        Unknown,
    }

    pub struct NmeaOutput {
        pub gll: i8,
        pub rmc: i8,
        pub vtg: i8,
        pub gga: i8,
        pub gsa: i8,
        pub gsv: i8,
    }

    pub struct EpoData {
        pub set: i8,
        pub fwn_ftow_week_number: i8,
        pub fwn_ftow_tow: i8,
        pub lwn_ltow_week_number: i8,
        pub lwn_ltow_tow: i8,
        pub fcwn_fctow_week_number: i8,
        pub fcwn_fctow_tow: i8,
        pub lcwn_lctow_week_number: i8,
        pub lcwn_lctow_tow: i8,
    }

    pub fn add_checksum(sentence: String) -> String {
        let mut checksum = 0;
        for char in sentence.as_bytes() {
            checksum ^= *char;
        }
        let checksum = format!("{:X}", checksum);  //Format as hexidecimal.
        let checksumed_sentence = format!("${}*{}\r\n", sentence, checksum).as_str().to_ascii_uppercase();
        return checksumed_sentence;
    }

    pub trait SendPmtk {
        fn send_command(&mut self, cmd: &str);
        // Just send it. bool for ack. true if not wanting ack.
        fn pmtk_001(&mut self, search_depth: i32) -> Pmtk001Ack;
        fn pmtk_500(&mut self) -> Option<String>;
        fn pmtk_startup(&mut self) -> bool;

        fn pmtk_101_cmd_hot_start(&mut self) -> bool;
        fn pmtk_102_cmd_warm_start(&mut self) -> bool;
        fn pmtk_103_cmd_cold_start(&mut self) -> bool;
        fn pmtk_104_cmd_full_cold_start(&mut self) -> bool;

        fn pmtk_220_set_nmea_updaterate(&mut self, update_rate: &str) -> Pmtk001Ack;
        fn pmtk_251_set_nmea_baudrate(&mut self, baud_rate: &str) -> Pmtk001Ack;

        fn pmtk_301_api_set_dgps_mode(&mut self, dgps_mode: &str) -> Pmtk001Ack;
        fn pmtk_401_api_q_dgps_mode(&mut self) -> DgpsMode;

        fn pmtk_313_api_set_sbas_enabled(&mut self, sbas: &str) -> Pmtk001Ack;
        fn pmtk_413_api_q_sbas_enabled(&mut self) -> Sbas;

        fn pmtk_314_api_set_nmea_output(&mut self, gll: i8, rmc: i8, vtg: i8, gga: i8, gsa: i8, gsv: i8, pmtkchn_interval: i8) -> Pmtk001Ack;
        fn pmtk_414_api_q_nmea_output(&mut self) -> NmeaOutput;

        fn pmtk_319_api_set_sbas_mode(&mut self, sbas_mode: &str) -> bool;
        fn pmtk_419_api_q_sbas_mode(&mut self) -> SbasMode;

        fn pmtk_605_q_release(&mut self) -> String;
        // fn pmtk_705_dt_release(&mut self) -> Pmtk001Ack;
        fn pmtk_607_q_epo_info(&mut self) -> EpoData;
        fn pmtk_127_cmd_clear_epo(&mut self) -> Pmtk001Ack;

        fn pmtk_397_set_nav_speed_threshold(&mut self, nav_threshold: f32) -> Pmtk001Ack;
        fn pmtk_386_set_nav_speed_threshold(&mut self, nav_threshold: f32) -> Pmtk001Ack;
        fn pmtk_447_q_nav_threshold(&mut self) -> f32;

        fn pmtk_161_cmd_standby_mode(&mut self) -> Pmtk001Ack;

        fn pmtk_223_set_al_dee_cfg(&mut self, sv: i8, snr: i8, ext_threshold: i32, ext_gap: i32) -> Pmtk001Ack;
        fn pmtk_225_cmd_periodic_mode(&mut self, run_type: u8, run_time: u32, sleep_time: u32,
                                      second_run_time: u32, second_sleep_time: u32) -> Pmtk001Ack;

        fn pmtk_286_cmd_aic_mode(&mut self, aic: bool) -> Pmtk001Ack;
        fn pmtk_869_cmd_easy_enable(&mut self, enable_easy: bool) -> Pmtk001Ack;
        fn pmtk_869_cmd_easy_query(&mut self) -> bool;

        fn pmtk_187_locus_config(&mut self, locus_interval: i8) -> Pmtk001Ack;

        fn pmtk_330_api_set_datum(&mut self, datum: u16) -> Pmtk001Ack;
        fn pmtk_430_api_q_datum(&mut self) -> u16;

        fn pmtk_351_api_set_support_qzss_nmea(&mut self, enable_qzss: bool) -> Pmtk001Ack;
        fn pmtk_352_api_set_stop_qzss(&mut self, enable: bool) -> Pmtk001Ack;
    }

    impl SendPmtk for Gps {
        #[allow(unused_must_use)]  // self.port.write is not used
        fn send_command(&mut self, cmd: &str) {
            let cmd = add_checksum(cmd.to_string());
            let byte_cmd = cmd.as_bytes();
            self.port.clear(serialport::ClearBuffer::Input);
            self.port.write(byte_cmd);
        }

        fn pmtk_001(&mut self, search_depth: i32) -> Pmtk001Ack {
            //! Format for this is $pmtk{cmd},{flag},{value}*{checksum}
    //! Value isn't always present.
            for _i in 0..search_depth {   // Check 10 lines before giving up.
                let line = self.read_line();
                if (&line[0..8] == "$PMTK001") && (is_valid_checksum(&line)) {
                    let line = line.trim();
                    // Remove checksum.
                    let line: Vec<&str> = line.split("*").collect();
                    let line: &str = line.get(0).unwrap();

                    let args: Vec<&str> = line.split(",").collect();
                    // args: $PMTK001, cmd, flag,
                    // let cmd: &str = args.get(1).expect("pmtk001 format not correct");
                    let flag: &str = args.get(2).expect("pmtk001 format not correct");
                    // let value: &str = args.get(3).unwrap_or(&"");

                    return if flag == "0" {
                        Pmtk001Ack::Invalid
                    } else if flag == "1" {
                        Pmtk001Ack::Unsupported
                    } else if flag == "2" {
                        Pmtk001Ack::Failed
                    } else if flag == "3" {
                        Pmtk001Ack::Success
                    } else {
                        Pmtk001Ack::NoPacket
                    };
                } else {
                    continue;
                }
            }
            return Pmtk001Ack::NoPacket;
        }

        fn pmtk_500(&mut self) -> Option<String> {
            //! 500 reply format if $PMTK500,arg,arg,arg.
    //! Return the string without checksum.
            for _i in 0..10 {   // Check 10 lines before giving up.
                let line = self.read_line();
                if (&line[0..5] == "$PMTK") && (is_valid_checksum(&line)) {
                    let line = line.trim();
                    // Remove checksum.
                    let line: Vec<&str> = line.split("*").collect();
                    let line: &str = line.get(0).unwrap();
                    return Some(line.to_string());
                }
            }
            return None;
        }

        fn pmtk_startup(&mut self) -> bool {
            //! Return true if it did reboot.
            // todo
            true
        }

        fn pmtk_101_cmd_hot_start(&mut self) -> bool {
            //! Hot restart gps: use all data in NV store
            self.send_command("PMTK101");
            self.pmtk_startup()
        }

        fn pmtk_102_cmd_warm_start(&mut self) -> bool {
            //! Warm restart gps: Dont use Ephemeris at re-start.
            self.send_command("PMTK102");
            self.pmtk_startup()
        }

        fn pmtk_103_cmd_cold_start(&mut self) -> bool {
            //! Cold restart gps: Don't use time, position, almanacs or Ephemeris data to restart.
            self.send_command("PMTK103");
            self.pmtk_startup()
        }

        fn pmtk_104_cmd_full_cold_start(&mut self) -> bool {
            //! Full restart gps: All systems, configs are reset. Basically factory reset.
            self.send_command("PMTK104");
            self.pmtk_startup()
        }

        fn pmtk_220_set_nmea_updaterate(&mut self, update_rate: &str) -> Pmtk001Ack {
            //! Set NMEA port update rate. Range is 100 to 10_000 miliseconds.
    //!
    //! Gets standard 001 response.
            self.send_command(format!("PMTK220,{}", update_rate).as_str());
            self.pmtk_001(10)
        }

        fn pmtk_251_set_nmea_baudrate(&mut self, baud_rate: &str) -> Pmtk001Ack {
            //! Set NMEA port baud rate: Setting are: 4800,9600,14400,19200,38400,57600,115200
    //!
    //! Probably 001 response.
            self.send_command(format!("PMTK251,{}", baud_rate).as_str());
            self.pmtk_001(10)
        }

        fn pmtk_301_api_set_dgps_mode(&mut self, dgps_mode: &str) -> Pmtk001Ack {
            //! Query DGPS mode.
    //! DGPS - Differential GPS.
    //! WAAS: Wide area augmentation system. Only avaliable in North America. Improved GPS
    //! accuracy by using a correction station.
    //!
    //! If you wish to set DGPS to RTCM, first set baud rate (pmtk_220_set_nmea_updaterate).
    //! RTCM: Not sure what this is.
    //!
    //! 001 reply.
            self.send_command(format!("PMTK301,{}", dgps_mode).as_str());
            self.pmtk_001(10)
        }

        fn pmtk_401_api_q_dgps_mode(&mut self) -> DgpsMode {
            //! Query what the DGPS setting is.
    //! Response is 501,{0,1,2} for {No DGPS, RTCM, WAAS}.
    //! Fn return: Option String 0, 1 or 2.

            self.send_command("PMTK401");

            // Should be just one arg.
            return match self.pmtk_500() {
                Some(args) => {
                    if args.len() != 10 {  // $PM TK5 01, {0,1,2}
                        return DgpsMode::Unknown;
                    }
                    let mode: String = args.chars().nth_back(0).unwrap().to_string();
                    let mode: &str = mode.as_str();
                    if mode == "0" {
                        return DgpsMode::NoDgps;
                    } else if mode == "1" {
                        DgpsMode::RTCM
                    } else if mode == "2" {
                        DgpsMode::WAAS
                    } else {
                        DgpsMode::Unknown
                    }
                }
                None => DgpsMode::Unknown
            };
        }

        fn pmtk_313_api_set_sbas_enabled(&mut self, sbas: &str) -> Pmtk001Ack {
            //! Standard 001 response.
            self.send_command(format!("PMTK313,{}", sbas).as_str());
            self.pmtk_001(10)
        }

        fn pmtk_413_api_q_sbas_enabled(&mut self) -> Sbas {
            //! $PMTK503,{0,1} for {disabled, enabled}
            self.send_command("PMTK413");
            return match self.pmtk_500() {
                Some(args) => {
                    if args.len() != 10 {
                        return Sbas::Unknown;
                    }
                    let mode = args.chars().nth_back(0).unwrap().to_string();
                    let mode = mode.as_str();
                    if mode == "0" {
                        Sbas::Disabled
                    } else if mode == "1" {
                        Sbas::Enabled
                    } else {
                        Sbas::Unknown
                    }
                }
                None => Sbas::Unknown
            };
        }

        fn pmtk_314_api_set_nmea_output(&mut self, gll: i8, rmc: i8, vtg: i8, gga: i8, gsa: i8, gsv: i8, pmtkchn_interval: i8) -> Pmtk001Ack {
            //! 19 fields can be parsed to this one.
    //! $PMTK314,{GPGLL},{GPRMC},{GPTVG},{GPGGA},{GPGAS},{GPGSV},{R}..6-17,{PMTKCHN interval}
    //! For each field, frequency setting is given: 0-5, 0-> Disabled,
    //! 1-> Output once everty one position fix, 2-> every second... every 5th.
    //! pmtk response is standard PMTK001
            // todo -- default is PMTK4314,-1*
            self.send_command(format!("PMTK314,{},{},{},{},{},{},0,0,0,0,0,0,0,{}",
                                      gll, rmc, vtg, gga, gsa, gsv, pmtkchn_interval).as_str());
            self.pmtk_001(10)
        }

        fn pmtk_414_api_q_nmea_output(&mut self) -> NmeaOutput {
            //! Return 514: PMTK514, the nmea outputs that are valid (see pmtk_314_api_set_nmea_output
    //! for the fields).
            self.send_command("PMTK414");
            return match self.pmtk_500() {
                Some(args) => {
                    let args: Vec<&str> = args.split(",").collect();
                    let gll: &str = args.get(1).unwrap_or(&"-1");
                    let rmc: &str = args.get(2).unwrap_or(&"-1");
                    let vtg: &str = args.get(3).unwrap_or(&"-1");
                    let gga: &str = args.get(4).unwrap_or(&"-1");
                    let gsa: &str = args.get(5).unwrap_or(&"-1");
                    let gsv: &str = args.get(6).unwrap_or(&"-1");

                    NmeaOutput {
                        gll: gll.parse::<i8>().unwrap(),
                        rmc: rmc.parse::<i8>().unwrap(),
                        vtg: vtg.parse::<i8>().unwrap(),
                        gga: gga.parse::<i8>().unwrap(),
                        gsa: gsa.parse::<i8>().unwrap(),
                        gsv: gsv.parse::<i8>().unwrap(),
                    }
                }
                None => NmeaOutput {
                    gll: -1,
                    rmc: -1,
                    vtg: -1,
                    gga: -1,
                    gsa: -1,
                    gsv: -1,
                }
            };
        }

        fn pmtk_319_api_set_sbas_mode(&mut self, sbas_mode: &str) -> bool {
            //! Set sbas mode. 0=testing mode and 1=integrity mode.
    //!
    //! Reboots itself.
            self.send_command(format!("PMTK391,{}", sbas_mode).as_str());
            self.pmtk_startup()
        }

        fn pmtk_419_api_q_sbas_mode(&mut self) -> SbasMode {
            //! 519 response, PMTK519,{0,1} for {testing mode, integrity mode}, set by 319.
    //! false is testing mode, true is integrity mode.
    //!
            self.send_command("PMTK419");
            return match self.pmtk_500() {
                Some(args) => {
                    let arg = args.chars().nth_back(0).unwrap().to_string();
                    let arg = arg.as_str();
                    if arg == "0" {
                        SbasMode::Testing
                    } else if arg == "1" {
                        SbasMode::Integrity
                    } else {
                        SbasMode::Unknown
                    }
                }
                None => SbasMode::Unknown
            };
        }

        fn pmtk_605_q_release(&mut self) -> String {
            //! $PMTK705,AXN_5.1.7_3333_19020118,0027,PA1010D,1.0*76
    //! Get firmware release info.
    //!
    //! Return blank string if no info found.
            self.send_command("PMTK605");
            return match self.pmtk_500() {
                Some(args) => {
                    args[8..args.len()].to_string()
                }
                None => "".to_string()
            };
        }

        fn pmtk_607_q_epo_info(&mut self) -> EpoData {
            //! $PMTK707,0,0,0,0,0,0,0,0,0*2E
    //! Get EPO data status
    //! 0 Set: Total number sets of EPO data stored in the GPS chip
    //! 1 FWN & FTOW : GPS week number
    //! 2 FWN & FTOW : TOW of the first set of EPO data stored in chip respectively
    //! 3 LWN & LTOW : GPS week number
    //! 4 LWN & LTOW : TOW of the last set of EPO data stored in chip respectively
    //! 5 FCWN & FCTOW : GPS week number
    //! 6 FCWN & FCTOW : TOW of the first set of EPO data that are currently used respectively
    //! 7 LCWN & LCTOW : GPS week number
    //! 8 LCWN & LCTOW : TOW of the last set of EPO data that are currently used respectively

            let args = self.pmtk_500().unwrap();
            let args: Vec<&str> = args.split(",").collect();
            EpoData {
                set: args.get(1).unwrap_or(&"-1").parse::<i8>().unwrap(),
                fwn_ftow_week_number: args.get(2).unwrap_or(&"-1").parse::<i8>().unwrap(),
                fwn_ftow_tow: args.get(3).unwrap_or(&"-1").parse::<i8>().unwrap(),
                lwn_ltow_week_number: args.get(4).unwrap_or(&"-1").parse::<i8>().unwrap(),
                lwn_ltow_tow: args.get(5).unwrap_or(&"-1").parse::<i8>().unwrap(),
                fcwn_fctow_week_number: args.get(6).unwrap_or(&"-1").parse::<i8>().unwrap(),
                fcwn_fctow_tow: args.get(7).unwrap_or(&"-1").parse::<i8>().unwrap(),
                lcwn_lctow_week_number: args.get(8).unwrap_or(&"-1").parse::<i8>().unwrap(),
                lcwn_lctow_tow: args.get(9).unwrap_or(&"-1").parse::<i8>().unwrap(),
            }
        }

        fn pmtk_127_cmd_clear_epo(&mut self) -> Pmtk001Ack {
            //! Response:
    //! Multiple $CLR,EPO,{000a8000}*5E lines, ending with a 001 response.
            self.send_command("PMTK127");
            self.pmtk_001(50) // 50 should be plenty. Probably.
        }

        fn pmtk_397_set_nav_speed_threshold(&mut self, nav_threshold: f32) -> Pmtk001Ack {
            //! Set nav speed threshold. If the speed calculated is lower than this threshold, outputed
    //! position is frozen.
    //! Nav Speed threshold: 0/ 0.2/ 0.4/ 0.6/ 0.8/ 1.0/1.5/2.0 (m/s)
    //! Disable:Nav Speed threshold is set to 0 m/sec.
    //! Standard 001 response.
    //!
    //! For MT3318 and MT3329 chips.
            self.send_command(format!("PMTK397,{:.1}", nav_threshold).as_str());
            self.pmtk_001(10)
        }

        fn pmtk_386_set_nav_speed_threshold(&mut self, nav_threshold: f32) -> Pmtk001Ack {
            //! Set nav speed threshold. If the speed calculated is lower than this threshold, outputed
    //! position is frozen.
    //! Nav Speed threshold: 0/ 0.2/ 0.4/ 0.6/ 0.8/ 1.0/1.5/2.0 (m/s)
    //! Disable:Nav Speed threshold is set to 0 m/sec.
    //! Standard 001 response.
    //!
    //! For MT3339 chips.
            self.send_command(format!("PMTK397,{:.1}", nav_threshold).as_str());
            self.pmtk_001(10)
        }

        fn pmtk_447_q_nav_threshold(&mut self) -> f32 {
            //! $PMTK527,0.40*04
    //! Gives current nav threshold. The range is 0/ 0.2/ 0.4/ 0.6/ 0.8/ 1.0/1.5/2.0 (m/s)
            self.send_command("PMTK447");
            return match self.pmtk_500() {
                Some(args) => {
                    let args: Vec<&str> = args.split(",").collect();
                    let nav_threshold: f32 = args.get(1).unwrap().parse::<f32>().unwrap();
                    nav_threshold
                }
                None => return -1.0
            };
        }

        fn pmtk_161_cmd_standby_mode(&mut self) -> Pmtk001Ack {
            //! Puts the gps to sleep PMTK161,0. Send anything to wake it back up.
    //! Standard 001 response.
            self.send_command("PMTK161,0");
            self.pmtk_001(10)
        }

        fn pmtk_223_set_al_dee_cfg(&mut self, sv: i8, snr: i8, ext_threshold: i32, ext_gap: i32) -> Pmtk001Ack {
            //! Should be used with the PMTK225 command to set periodic mode.
    //!
    //! SV: Default 1, range 1-4. Increases the time to receive more ephemeris data while the
    //! number of satellites without ephemeris data is less than the SV value.
    //!
    //! SNR: Fedault 30, range 25-30. Enable receiving ephemeris data while the SNR of satellite
    //! is more than the value.
    //!
    //! Extention threshold (millisecond): default 180_000, range 40_000-180_000. The extension time
    //! for ephemeris data receiving.
    //!
    //! Extention gap: Default 60000, range 0-3_600_000
    //!
    //! Standard 001 response.
            self.send_command(format!("PMTK223,{},{},{},{}", sv, snr, ext_threshold, ext_gap).as_str());
            self.pmtk_001(10)
        }

        fn pmtk_225_cmd_periodic_mode(&mut self, run_type: u8, run_time: u32, sleep_time: u32,
                                      second_run_time: u32, second_sleep_time: u32) -> Pmtk001Ack {
            //! Enter standby or backup mode for power saving.
    //! PMTK225,Type,Run time,Sleep time, Second run time,Second sleep time
    //! run_type: operation mode
    //!     ‘0’ = go back to normal mode
    //!     ‘1’ = Periodic backup mode
    //!     ‘2’ = Periodic standby mode
    //!     ‘4’ = Perpetual backup mode
    //!     ‘8’ = AlwaysLocateTM standby mode
    //!     ‘9’ = AlwaysLocateTM backup mode
    //! Run time (millisecond): Duration to fix for (or attempt to fix for) before switching
    //! from running modeback to a minimum power sleep mode.
    //!     '0’: disable
    //!     >=’1,000’: enable Range: 1,000~518400000
    //! Sleep time (millisecond):Interval to come out of a minimum power sleep mode and start
    //! running in order to get a new position fix.
    //!     ‘0’: disable
    //!     >=’1,000’: enable Range: 1,000~518400000
    //! Second run time (millisecond): Duration to fix for (or attempt to fix for) before
    //! switching from running mode back to a minimum power sleep mode.
    //!     ‘0’: disable
    //!     >=’1,000’: enable Range: 1,000~518400000
    //! Second sleep time (millisecond): Interval to come out of a minimum power sleep mode and
    //! start running in order to get a new position fix.
    //!     ‘0’: disable
    //!     >=’1,000’: enable Range: 1,000~518400000
    //!
    //! Note：1.The second run time should larger than first run time when non-zero value.
    //! 2.The purpose of second run time and sleep time can let module to catch more satellite
    //!     ephemeris data in cold boot condition. The value of them can be null. Then it will
    //!     use the first run time and sleep time for ephemeris data receiving.
    //! 3.AlwaysLocateTM is an intelligent controller of MT3339 power saving mode. Depending on
    //!     the environment and motion conditions, MT3339 can adaptive adjust the on/off time
    //!     to achieve balance of positioning accuracy and power consumption.
    //! 4.This command needs to work normal with some hardware circuits.
    //!
    //! Reboot response then standard 001 reply.

            self.send_command(format!("PMTK223,{},{},{},{},{}",
                                      run_type, run_time, sleep_time, second_run_time, second_sleep_time).as_str());
            self.pmtk_001(10)
        }

        fn pmtk_286_cmd_aic_mode(&mut self, aic: bool) -> Pmtk001Ack {
            //! Active Interference Cancellation provides effective narrow-band interference and
    //! jamming elimination.
    //!
    //! aic: true is enable, false is disable.
    //! Standard 001 response.
            if aic {
                self.send_command("PMTK286,1")
            } else {
                self.send_command("PMTK286,0")
            }
            self.pmtk_001(10)
        }

        fn pmtk_869_cmd_easy_enable(&mut self, enable_easy: bool) -> Pmtk001Ack {
            //! Enable or disable EASY function.
    //! Enabled by default.
    //! Requires VBACKUP pin to be connected to battery.
    //! Only valid for 1Hz update rate
    //!
    //! true is enable easy, false is disable.
    //!
    //! If you wish to query the EASY function, use pmtk_869_cmd_easy_query
    //! Response
    //! pmtk,0 -> gives $PMTK869,2,1,3*29
    //! pmtk,1,0 -> Gives 001 reply.
    //! pmtk,2,{0,1} -> Gives 001 reply.
            if enable_easy {
                self.send_command("PMTK869,1,1")
            } else {
                self.send_command("PMTK869,1,0")
            }
            self.pmtk_001(10)
        }

        fn pmtk_869_cmd_easy_query(&mut self) -> bool {
            //! Query the EASY command status. Return true or false, true is enabled, false it disabled.
    //!
    //! Response: $PMTK869,2,{0:disabled, 1:enabled},{001 status}
            self.send_command("PMTK869,0");
            return match self.pmtk_500() {
                Some(args) => {
                    let args: Vec<&str> = args.split(",").collect();
                    if args.get(2).unwrap() == &"0" {
                        false
                    } else {
                        true
                    }
                }
                None => true
            };
        }

        fn pmtk_187_locus_config(&mut self, locus_interval: i8) -> Pmtk001Ack {
            //! Configure Locus setting.
    //! Locus mode (1 for interval mode) is always on.
    //! Interval, in seconds, is how often to log a data.
    //!
    //! Standard 001 reply.
            self.send_command(format!("PMTK187,1,{}", locus_interval).as_str());
            self.pmtk_001(10)
        }

        fn pmtk_330_api_set_datum(&mut self, datum: u16) -> Pmtk001Ack {
            //! Configure Datum. 222 datum options.
    //! ‘0’ = WGS84
    //! ‘1’ = TOKYO-M
    //! ‘2’ = TOKYO-A
    //!
    //! A full list is on the GTOP Datum list, but I can't find it.
    //! Standard 001 reply.
            self.send_command(format!("PMTK330,{}", datum).as_str());
            self.pmtk_001(10)
        }

        fn pmtk_430_api_q_datum(&mut self) -> u16 {
            //! Query current datum. Gives PMTK530,datum
    //! See pmtk_330_api_set_datum for more details on datum.
    //!
    //! 0 is return value if there is an error.
            self.send_command("PMTK430");
            return match self.pmtk_500() {
                Some(args) => {
                    let args: Vec<&str> = args.split(",").collect();
                    let datum = args.get(1).unwrap_or(&"0").parse::<u16>().unwrap();
                    datum
                }
                None => 0
            };
        }

        fn pmtk_351_api_set_support_qzss_nmea(&mut self, enable_qzss: bool) -> Pmtk001Ack {
            //! Sets the output to be the QZSS NMEA format.
    //! True is enable, false is disable. Default is disable.
    //!
    //! Standard 001 reply.
            if enable_qzss {
                self.send_command("PMTK351,1")
            } else {
                self.send_command("PMTK351,0")
            }
            self.pmtk_001(10)
        }

        fn pmtk_352_api_set_stop_qzss(&mut self, enable: bool) -> Pmtk001Ack {
            //! Since QZSS is regional positioning service. The command allow user enable or disable QZSS function.
    //! Default is enable QZSS function
    //!
    //! Enable is true, disable is false. Default is enable.
    //! Standard 001 reply.
            if enable {
                self.send_command("PMTK352,0")
            } else {
                self.send_command("PMTK352,1")
            }
            self.pmtk_001(10)
        }
    }
}

#[cfg(test)]
mod pmtktests {
    use super::send_pmtk::add_checksum;

    #[test]
    fn checksum() {
        assert_eq!(add_checksum("GNGGA,165419.000,5132.7378,N,00005.9192,W,1,7,1.93,34.4,M,47.0,M,,".to_string()), "$GNGGA,165419.000,5132.7378,N,00005.9192,W,1,7,1.93,34.4,M,47.0,M,,*6A\r\n".to_string());
        assert_eq!(add_checksum("PMTK103".to_string()), "$PMTK103*30\r\n")
    }


}
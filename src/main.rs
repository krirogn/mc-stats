use std::{
    collections::{BTreeMap, HashMap},
    env,
    fs::{self, File},
    io::{BufReader, Read},
};

use chrono::{Datelike, NaiveDate, NaiveDateTime, Utc};
use cli_table::{Cell, CellStruct, Style, Table};
use flate2::bufread::GzDecoder;

#[derive(Debug, Clone)]
struct PlayerLog {
    player: String,
    time: NaiveDateTime,
    connect: bool,
}

fn main() {
    // Get cli args
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("You must supply one path argument to the logs folder");
    }

    // Get all files
    let path = args.get(1).expect("Couldn't get the path CLI argument");
    let mut files = fs::read_dir(&path)
        .expect("Couldn't open the log directory")
        .into_iter()
        .map(|x| {
            x.expect("Couldn't get file")
                .file_name()
                .to_str()
                .expect("Couldn't get file name")
                .to_owned()
        })
        .filter(|x| x[..4].parse::<u16>().is_ok() || x.starts_with("latest"))
        .collect::<Vec<String>>();

    files.sort();

    // Go through files and pare player connections
    let mut player_entries: Vec<PlayerLog> = Vec::new();
    for file_name in files {
        let mut log_file =
            File::open(format!("{}{}", path, file_name)).expect("Coudln't open log file");

        let mut s = String::new();
        if file_name.ends_with(".gz") {
            // Decompress GZ files
            let log_buf = BufReader::new(log_file);
            let mut gz = GzDecoder::new(log_buf);
            gz.read_to_string(&mut s).expect("Couldn't decompress log");
        } else {
            log_file.read_to_string(&mut s).expect("Couldn't read log");
        }

        let log = s.split("\n").filter(|&i| i != "").collect::<Vec<&str>>();

        for line in log {
            // Log in
            if line.contains("logged in with") {
                // Player name
                let player_name = line
                    .split(": ")
                    .collect::<Vec<&str>>()
                    .get(1)
                    .unwrap()
                    .split("[")
                    .collect::<Vec<&str>>()
                    .get(0)
                    .unwrap()
                    .to_owned();

                // Time
                let time: NaiveDateTime;
                if file_name.starts_with("latest") {
                    let ndt = Utc::now();

                    let year = ndt.year();
                    let month = ndt.month();
                    let day = ndt.day();

                    let hour = line[1..=2].parse::<u32>().unwrap();
                    let minute = line[4..=5].parse::<u32>().unwrap();
                    let seconds = line[7..=8].parse::<u32>().unwrap();

                    time = NaiveDate::from_ymd_opt(year, month, day)
                        .unwrap()
                        .and_hms_opt(hour, minute, seconds)
                        .unwrap();
                } else {
                    let file_date = file_name[..10].split("-").collect::<Vec<&str>>();
                    let year = file_date.get(0).unwrap().parse::<i32>().unwrap();
                    let month = file_date.get(1).unwrap().parse::<u32>().unwrap();
                    let day = file_date.get(2).unwrap().parse::<u32>().unwrap();

                    let hour = line[1..=2].parse::<u32>().unwrap();
                    let minute = line[4..=5].parse::<u32>().unwrap();
                    let seconds = line[7..=8].parse::<u32>().unwrap();

                    time = NaiveDate::from_ymd_opt(year, month, day)
                        .unwrap()
                        .and_hms_opt(hour, minute, seconds)
                        .unwrap();
                }

                // Add entry
                player_entries.push(PlayerLog {
                    player: player_name.to_string(),
                    time,
                    connect: true,
                });
            // Log out
            } else if line.contains("lost connection") {
                let mut player_name = line
                    .split(": ")
                    .collect::<Vec<&str>>()
                    .get(1)
                    .unwrap()
                    .split(" ")
                    .collect::<Vec<&str>>()
                    .get(0)
                    .unwrap()
                    .to_owned();

                if player_name.starts_with("com.mojang.authlib.GameProfile@") {
                    player_name = player_name
                        .split("=")
                        .collect::<Vec<&str>>()
                        .get(2)
                        .unwrap()
                        .split(",")
                        .collect::<Vec<&str>>()
                        .get(0)
                        .unwrap();
                }

                // Time
                let time: NaiveDateTime;
                if file_name.starts_with("latest") {
                    let ndt = Utc::now();

                    let year = ndt.year();
                    let month = ndt.month();
                    let day = ndt.day();

                    let hour = line[1..=2].parse::<u32>().unwrap();
                    let minute = line[4..=5].parse::<u32>().unwrap();
                    let seconds = line[7..=8].parse::<u32>().unwrap();

                    time = NaiveDate::from_ymd_opt(year, month, day)
                        .unwrap()
                        .and_hms_opt(hour, minute, seconds)
                        .unwrap();
                } else {
                    let file_date = file_name[..10].split("-").collect::<Vec<&str>>();
                    let year = file_date.get(0).unwrap().parse::<i32>().unwrap();
                    let month = file_date.get(1).unwrap().parse::<u32>().unwrap();
                    let day = file_date.get(2).unwrap().parse::<u32>().unwrap();

                    let hour = line[1..=2].parse::<u32>().unwrap();
                    let minute = line[4..=5].parse::<u32>().unwrap();
                    let seconds = line[7..=8].parse::<u32>().unwrap();

                    time = NaiveDate::from_ymd_opt(year, month, day)
                        .unwrap()
                        .and_hms_opt(hour, minute, seconds)
                        .unwrap();
                }

                player_entries.push(PlayerLog {
                    player: player_name.to_string(),
                    time,
                    connect: false,
                });
            }
        }
    }

    // Get players times
    let mut players: HashMap<String, i64> = HashMap::new();
    let mut logins: HashMap<String, PlayerLog> = HashMap::new();
    for entry in player_entries {
        // Login
        if entry.connect {
            if !logins.contains_key(&entry.player) {
                logins.insert(entry.player.clone(), entry.clone());
            }
        // Logout
        } else {
            if logins.contains_key(&entry.player) {
                // Get difference
                let old_time = logins.get(&entry.player).expect("Couldn't get player").time;
                let new_time = entry.time;
                let elapsed_time = new_time - old_time;

                if players.contains_key(&entry.player) {
                    let duration = players
                        .get(&entry.player)
                        .expect("Couldn't get player HashMap")
                        .to_owned();

                    players.insert(entry.player.clone(), duration + elapsed_time.num_seconds());
                } else {
                    players.insert(entry.player.clone(), elapsed_time.num_seconds());
                }

                logins.remove_entry(&entry.player);
            }
        }
    }

    // Handle people that are currently online
    for login in logins {
        // Get difference
        let old_time = login.1.time;
        let new_time = Utc::now().naive_utc();
        let elapsed_time = new_time - old_time;

        let duration = players
            .get(&login.1.player)
            .expect("Couldn't get player HashMap")
            .to_owned();

        players.insert(login.0, duration + elapsed_time.num_seconds());
    }

    // Convert the secons played in `players` into a readable format and return
    let mut table: Vec<Vec<CellStruct>> = Vec::new();
    let players: BTreeMap<i64, String> = players
        .iter()
        .map(|(k, v)| (v.to_owned(), k.to_owned()))
        .collect();
    for player in players.iter().rev() {
        let seconds = player.0 % 60;
        let minutes = (player.0 / 60) % 60;
        let hours = ((player.0 / 60) / 60) % 24;
        let days = ((player.0 / 60) / 60) / 24;

        table.push(vec![
            player.1.cell(),
            format!("{:0>2}:{:0>2}:{:0>2}:{:0>2}", days, hours, minutes, seconds)
                .cell()
                .justify(cli_table::format::Justify::Right),
        ]);
    }

    let cli_table = table
        .table()
        .title(vec![
            "Player".cell().bold(true),
            "Time DD:HH:MM:SS".cell().bold(true),
        ])
        .bold(true);
    println!("{}", cli_table.display().expect("Couldn't display table"));
}

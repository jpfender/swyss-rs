use clap::Clap;
use std::cell::RefCell;
use std::fs;
use std::io;
use std::io::Write;
use std::path::Path;
use std::process::exit;
use std::process::Command;
use std::rc::Rc;
use std::thread;
use swyss::*;

#[derive(Clap)]
struct Opts {
    #[clap(short, long, parse(from_occurrences))]
    img: i32,
    file: String,
}

/// Prompts and reads the score for a single player from the command line. Inputs that can't be
/// parsed into scores are rejected immediately, while inputs that are valid integers but invalid
/// scores will be rejected by the pairing after both scores have been entered.
fn read_score(num: u32, name: &String) -> Result<u8, String> {
    print!("[{}] {} > ", num, name);
    io::stdout().flush().unwrap();

    let mut score = String::new();

    match io::stdin().read_line(&mut score) {
        Ok(_) => {}
        Err(_) => return Err(String::from("Could not read input!")),
    };

    let score = match score.trim().parse() {
        Ok(s) => s,
        Err(_) => return Err(String::from("Could not parse score into integer!")),
    };

    Ok(score)
}

pub fn main() -> io::Result<()> {
    let opts = Opts::parse();

    let filename = opts.file;

    let img = match opts.img {
        0 => false,
        _ => true,
    };

    let mut players: Vec<Rc<RefCell<Player>>> = Vec::new();

    if img {
        let files = fs::read_dir(filename)?
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, io::Error>>()?;

        for f in files {
            let os_str = f.into_os_string();
            let name = os_str.into_string().unwrap();
            let p = Rc::new(RefCell::new(Player::new(&name)));
            players.push(p);
        }
    } else {
        let contents = match fs::read_to_string(filename) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("{}", e);
                exit(1);
            }
        };

        for line in contents.lines() {
            // Maybe we could add checks for duplicate entries here
            let p = Rc::new(RefCell::new(Player::new(line.trim())));
            players.push(p);
        }
    }

    let mut tourn = Tournament::new(players);

    while let Some(pairing) = tourn.next_round() {
        println!(
            "\n\n=== ROUND {}/{} ===\n",
            tourn.current_round, tourn.rounds
        );

        for pair in &pairing {
            let mut read = true;

            let uuid = pair.0;

            let home_file = String::from(&pair.1);
            let away_file = String::from(&pair.2);

            let home = String::from(Path::new(&home_file).file_stem().unwrap().to_str().unwrap());
            let away = String::from(Path::new(&away_file).file_stem().unwrap().to_str().unwrap());

            thread::spawn(|| {
                Command::new("feh")
                    .arg("-g")
                    .arg("960x1080+0+0")
                    .arg(home_file)
                    .arg("--scale-down")
                    .arg("--title")
                    .arg("1")
                    .output()
                    .expect("failed to execute process");
            });

            thread::spawn(|| {
                Command::new("feh")
                    .arg("-g")
                    .arg("960x1080+1920+0")
                    .arg(away_file)
                    .arg("--scale-down")
                    .arg("--title")
                    .arg("2")
                    .output()
                    .expect("failed to execute process");
            });

            while read {
                println!("\nPAIRING:\n[1] {}\n[2] {}\n", home, away);

                let home_score = match read_score(1, &home) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("{}", e);
                        continue;
                    }
                };

                let away_score = match read_score(2, &away) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("{}", e);
                        continue;
                    }
                };

                let mut drawn = 0;
                if home_score == 1 && away_score == 1 {
                    drawn = 1;
                }

                // `end_match()` returns an `Err` if the scores were invalid, in which case we do
                // not set `read` to `false`, resulting in another round
                match tourn.end_match(uuid, home_score, away_score, drawn) {
                    Ok(_) => read = false,
                    Err(e) => eprintln!("Error recording result: {}", e),
                };
            }

            Command::new("killall")
                .arg("feh")
                .output()
                .expect("failed to kill feh");
        }
    }

    let players = tourn.ranking();

    println!("\n=== RESULTS ===\n");

    println!("Rank\tName\t\tMP\tOMWP\tGWP\tOGWP");
    println!("----\t----\t\t--\t----\t---\t----");
    let mut rank = 1;
    for p in &players {
        let p = p.borrow();
        println!(
            "{}.\t{}\t\t{}\t{:.2}\t{:.2}\t{:.2}",
            rank,
            p.name,
            p.match_points,
            p.opponents_match_win_percentage(),
            p.game_win_percentage(),
            p.opponents_game_win_percentage()
        );
        rank += 1;
    }

    Ok(())
}

use std::convert::TryInto;

use rand::{thread_rng, Rng};
use shoshodamon::{
    ban2::Ban2 as Ban,
    evaluator::{self, alpha_beta2},
    Hand,
};

fn main() {
    // let ban =
    //     Ban::from_sfen("lnsg2k2/6G2/ppp6/3p+R4/9/9/PPPPPPP1P/1B7/LNSGKGSNL w RBNL6Ps 1").unwrap();

    // let possibes = ban.get_possibility_bans(ban.turn);
    // eval(&ban, 100);
    // for item in possibes {
    //     println!(
    //         "{} {:?} {}",
    //         item.0.to_sfen(),
    //         item.1.clone(),
    //         String::from(item.1)
    //     );
    // }

    let mut current_ban = None;
    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();

        let args = input.split_ascii_whitespace().collect::<Vec<_>>();
        match args[0] {
            "usi" => {
                println!("id name Shoshodamon v0.0.1");
                println!("id author MysteryJump");
                println!("usiok");
            }
            "setoption" => {}
            "usinewgame" => {}
            "isready" => {
                println!("readyok");
            }
            "position" => {
                let mut args = &args[1..];
                let sp = if args[0] == "startpos" {
                    args = &args[1..];
                    shoshodamon::START_POS.to_string()
                } else if args[0] == "sfen" {
                    args = &args[1..];
                    let sfen = format!("{} {} {} {}", args[0], args[1], args[2], args[3]);
                    args = &args[4..];
                    sfen
                } else {
                    panic!();
                };
                let mut ban = Ban::from_sfen(&sp).unwrap();
                if args.is_empty() {
                    current_ban = Some(ban);
                    continue;
                }
                if args[0] != "moves" {
                    panic!();
                } else {
                    args = &args[1..];
                }

                for mv in args {
                    let hand = (*mv).try_into().unwrap();
                    match hand {
                        Hand::Movement {
                            x,
                            y,
                            dx,
                            dy,
                            with_promote,
                        } => ban
                            .move_piece(
                                x,
                                y,
                                (x as isize + dx) as usize,
                                (y as isize + dy) as usize,
                                with_promote,
                            )
                            .unwrap(),
                        Hand::Putting { piece, x, y } => ban.put_piece(piece, x, y).unwrap(),
                    }
                }
                current_ban = Some(ban);
            }
            "go" => {
                if let Some(ban) = current_ban.clone() {
                    // let depth = 1000000;
                    let result = alpha_beta2(&ban, Vec::new(), -50000, 50000, 5, true); // eval(&ban, depth);
                    let depth = evaluator::COUNT.load(std::sync::atomic::Ordering::Relaxed);
                    evaluator::COUNT.store(0, std::sync::atomic::Ordering::Release);
                    if let Some(r) = result {
                        let hand = &r.0[0];
                        println!(
                            "info depth {} nodes {} score cp {} pv {}",
                            r.0.len(),
                            depth,
                            r.1,
                            r.0.iter()
                                .map(|x| String::from(x.clone()))
                                .collect::<Vec<_>>()
                                .join(" ")
                        );
                        println!("bestmove {}", String::from(hand.clone()))
                    } else {
                        let bans = ban.get_possibility_bans(ban.turn);
                        if bans.is_empty() {
                            println!("bestmove resign")
                        } else {
                            let ran = thread_rng().gen_range(0..bans.len());
                            let hand = bans[ran].1.clone();
                            println!("bestmove {}", String::from(hand));
                        }
                    }
                } else {
                    panic!()
                }
            }
            "gameover" => {
                current_ban = None;
            }
            _ => panic!(),
        }
    }
}

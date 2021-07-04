use std::convert::TryInto;

use rand::{thread_rng, Rng};
use shoshodamon::{evaluator::eval, Ban, Hand, Piece};

const START_POS: &str = "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1";

#[allow(dead_code)]
fn test() {
    let mut s = shoshodamon::Ban::parse_sfen(START_POS).unwrap();
    // println!("{}", s.to_sfen());
    s.move_piece(1, 6, 0, -1, false).unwrap();
    println!("{}", s.to_sfen());
    s.move_piece(7, 2, 0, 1, false).unwrap();
    println!("{}", s.to_sfen());
    s.move_piece(1, 5, 0, -1, false).unwrap();
    println!("{}", s.to_sfen());
    s.move_piece(7, 3, 0, 1, false).unwrap();
    println!("{}", s.to_sfen());
    s.move_piece(5, 8, 1, -1, false).unwrap();
    println!("{}", s.to_sfen());
    s.move_piece(3, 0, -1, 1, false).unwrap();
    println!("{}", s.to_sfen());
    s.move_piece(1, 4, 0, -1, false).unwrap();
    println!("{}", s.to_sfen());
    s.move_piece(1, 2, 0, 1, false).unwrap();
    println!("{}", s.to_sfen());
    s.move_piece(1, 7, 0, -4, false).unwrap();
    println!("{}", s.to_sfen());
    s.put_piece(Piece::Fu, 1, 2).unwrap();
    println!("{}", s.to_sfen());
    s.move_piece(1, 3, 0, -1, true).unwrap();
    println!("{}", s.to_sfen());
    s.move_piece(8, 0, 0, 1, false).unwrap();
    println!("{}", s.to_sfen());
    s.move_piece(1, 2, 1, -1, false).unwrap();
    println!("{}", s.to_sfen());
    s.move_piece(7, 1, 0, 2, false).unwrap();
    println!("{}", s.to_sfen());
    s.move_piece(2, 1, 0, -1, false).unwrap();
    println!("{}", s.to_sfen());

    // s.move_piece(0, 0, 0, 1, false).unwrap();
    let count = s.get_possibilities_ban(*s.turn());
    println!("{}", count.len());
    for item in count {
        println!("{}", item.0.to_sfen());
    }
    // s.move_piece(2, 0, 1, 0, false).unwrap();
    println!("{} {}", s.is_check(true), s.is_check(false));
    // s.move_piece(x, y, dx, dy, with_promote)
    // println!("{:#?}", s);
    println!("{}", s.to_sfen());
}

fn main() {
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
                    START_POS.to_string()
                } else if args[0] == "sfen" {
                    let sfen = format!("{} {} {} {}", args[0], args[1], args[2], args[3]);
                    args = &args[4..];
                    sfen
                } else {
                    panic!();
                };
                let mut ban = Ban::parse_sfen(&sp).unwrap();
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
                        } => ban.move_piece(x, y, dx, dy, with_promote).unwrap(),
                        Hand::Putting { piece, x, y } => ban.put_piece(piece, x, y).unwrap(),
                    }
                }
                current_ban = Some(ban);
            }
            "go" => {
                if let Some(ban) = current_ban.clone() {
                    let result = eval(&ban, 100000);
                    if let Some(r) = result {
                        let hand = &r.0[0];
                        println!("bestmove {}", String::from(hand.clone()))
                    } else {
                        let bans = ban.get_possibilities_ban(*ban.turn());
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
    // test()
}

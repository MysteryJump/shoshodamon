use std::{
    cmp::max,
    collections::LinkedList,
    sync::{
        atomic::{AtomicI32, Ordering},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

use crate::{ban2::Ban2 as Ban, Hand, Piece};

struct BanBeforeHands(Ban, Vec<Hand>);

pub fn eval(ban: &Ban, max_count: i32) -> Option<(Vec<Hand>, i32)> {
    let possiblities = ban.get_possibility_bans(ban.turn);
    if possiblities.is_empty() || ban.is_check_mate(ban.turn) {
        return None;
    }
    let thread_count = 8;
    let mut threads = Vec::new();
    let (sender, reader) = crossbeam::channel::unbounded::<BanBeforeHands>();
    let moves = Arc::new(Mutex::new(LinkedList::new()));
    let count = Arc::new(AtomicI32::new(0));

    for _ in 0..thread_count {
        let reader = reader.clone();
        let sender = sender.clone();
        let moves = moves.clone();
        let count = count.clone();
        threads.push(thread::spawn(move || {
            while let Ok(mes) = reader.recv_timeout(Duration::from_millis(1000)) {
                let possiblities = mes.0.get_possibility_bans(mes.0.turn);
                if possiblities.is_empty() {
                    break;
                }
                for possib in &possiblities {
                    let mut mes2 = mes.1.clone();
                    mes2.push(possib.1.clone());
                    let score = get_evaluated_value(&possib.0);
                    {
                        let mut locked = moves.lock().unwrap();
                        locked.push_front((mes2.clone(), score));
                        if locked.len() > 100000 {
                            locked.pop_back();
                        }
                        count.fetch_add(1, Ordering::Relaxed);
                    }
                    sender.send(BanBeforeHands(possib.0.clone(), mes2)).unwrap();
                }
                if count.load(Ordering::Relaxed) > max_count {
                    break;
                }
            }
        }))
    }
    for (ban, hand) in possiblities {
        let pair = BanBeforeHands(ban, vec![hand]);
        sender.send(pair).unwrap();
    }

    for thread in threads {
        thread.join().unwrap();
    }

    let locked = moves.lock().unwrap();
    let collected = locked.iter().collect::<Vec<_>>();
    let max_depth = collected.iter().max_by(|x, y| x.0.len().cmp(&y.0.len()));
    let depth = if let Some((v, _)) = max_depth {
        v.len() as isize
    } else {
        return None;
    };
    let min_depth = max(0, depth - 2);
    let mut filtered = collected
        .iter()
        .filter(|x| x.0.len() >= min_depth as usize)
        .collect::<Vec<_>>();
    filtered.sort_unstable_by_key(|x| x.1);
    println!("{}", filtered.len());
    println!(
        "info string Front: {}, Back: {}",
        filtered[0].1,
        filtered.last().unwrap().1
    );
    let result = if !ban.turn {
        *filtered[0]
    } else {
        *filtered.last().unwrap()
    };
    Some((result.0.clone(), result.1))
}

fn get_evaluated_value(ban: &Ban) -> i32 {
    let mut primary = 0;
    let mut secondary = 0;

    for i in 1..=9 {
        for j in 1..=9 {
            if let Some(piece) = ban.get_position(i, j) {
                let score = get_score(&(piece.piece, piece.promoted));
                if piece.turn {
                    primary += score;
                } else {
                    secondary += score;
                }
            }
        }
    }

    primary += ban
        .primary_pieces
        .iter()
        .fold(0, |bef, cur| bef + get_score(&(*cur, false)));
    secondary += ban
        .secondary_pieces
        .iter()
        .fold(0, |bef, cur| bef + get_score(&(*cur, false)));
    primary - secondary
}

fn get_score((piece, promoted): &(Piece, bool)) -> i32 {
    match (piece, promoted) {
        (Piece::Fu, true) => 3,
        (Piece::Fu, false) => 1,
        (Piece::Ou, _) => 0,
        (Piece::Kaku, true) => 47,
        (Piece::Kaku, false) => 40,
        (Piece::Hisha, true) => 50,
        (Piece::Hisha, false) => 45,
        (Piece::Kin, _) => 15,
        (Piece::Gin, true) => 10,
        (Piece::Gin, false) => 10,
        (Piece::Keima, true) => 5,
        (Piece::Keima, false) => 6,
        (Piece::Kyosha, true) => 4,
        (Piece::Kyosha, false) => 5,
    }
}

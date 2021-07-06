#![allow(clippy::needless_range_loop)]
#![allow(clippy::result_unit_err)]
#![feature(bindings_after_at)]

pub mod ban2;
pub mod evaluator;

use std::convert::{TryFrom, TryInto};
/*
(0-indexed)
x <-----------
              |
              |
              |
              |
              |
            ∨
              y
*/

pub const START_POS: &str = "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1";

#[derive(Debug, Clone, Copy)]
pub struct OnBoardPiece {
    piece: Piece,
    promoted: bool,
    turn: bool,
}

impl From<&OnBoardPiece> for String {
    fn from(value: &OnBoardPiece) -> Self {
        let pp = PieceBoolPair(value.piece, value.turn);
        let ch: char = <PieceBoolPair>::into(pp);
        let mut ch = ch.to_string();
        if value.promoted {
            ch.insert(0, '+');
        }
        ch
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Piece {
    Fu,
    Ou,
    Kaku,
    Hisha,
    Kin,
    Gin,
    Keima,
    Kyosha,
}

impl Piece {
    // get near piece movement does not contains kyousha, kaku, hisha straight movement
    const fn get_near_piece_movement(&self, turn: bool, promoted: bool) -> &[(isize, isize); 8] {
        let full_movement = &[
            (1, 0),
            (1, 1),
            (0, 1),
            (-1, 1),
            (-1, 0),
            (-1, -1),
            (0, -1),
            (1, -1),
        ];
        let kin_movement = &[
            (0, -1),
            (0, 1),
            (1, 0),
            (-1, 0),
            (1, -1),
            (-1, -1),
            (0, 0),
            (0, 0),
        ];
        let rev_kin_movement = &[
            (0, -1),
            (0, 1),
            (1, 0),
            (-1, 0),
            (1, 1),
            (-1, 1),
            (0, 0),
            (0, 0),
        ];
        let invalid_movement = &[
            (0, 0),
            (0, 0),
            (0, 0),
            (0, 0),
            (0, 0),
            (0, 0),
            (0, 0),
            (0, 0),
        ];
        let fu_movement = &[
            (0, -1),
            (0, 0),
            (0, 0),
            (0, 0),
            (0, 0),
            (0, 0),
            (0, 0),
            (0, 0),
        ];
        let rev_fu_movement = &[
            (0, 1),
            (0, 0),
            (0, 0),
            (0, 0),
            (0, 0),
            (0, 0),
            (0, 0),
            (0, 0),
        ];
        let kaku_movement = &[
            (1, 1),
            (-1, -1),
            (1, -1),
            (-1, 1),
            (0, 0),
            (0, 0),
            (0, 0),
            (0, 0),
        ];
        let hisha_movement = &[
            (1, 0),
            (0, 1),
            (-1, 0),
            (0, -1),
            (0, 0),
            (0, 0),
            (0, 0),
            (0, 0),
        ];
        let gin_movement = &[
            (0, -1),
            (-1, -1),
            (1, -1),
            (1, 1),
            (-1, 1),
            (0, 0),
            (0, 0),
            (0, 0),
        ];
        let rev_gin_movement = &[
            (0, 1),
            (-1, 1),
            (1, 1),
            (1, -1),
            (-1, -1),
            (0, 0),
            (0, 0),
            (0, 0),
        ];
        let keima_movement = &[
            (-1, -2),
            (1, -2),
            (0, 0),
            (0, 0),
            (0, 0),
            (0, 0),
            (0, 0),
            (0, 0),
        ];
        let rev_keima_movement = &[
            (-1, 2),
            (1, 2),
            (0, 0),
            (0, 0),
            (0, 0),
            (0, 0),
            (0, 0),
            (0, 0),
        ];

        match (self, turn, promoted) {
            (Piece::Fu, true, true) | (Piece::Kyosha, true, true) => kin_movement,
            (Piece::Fu, true, false) | (Piece::Kyosha, true, false) => fu_movement,
            (Piece::Fu, false, true) | (Piece::Kyosha, false, true) => rev_kin_movement,
            (Piece::Fu, false, false) | (Piece::Kyosha, false, false) => rev_fu_movement,
            (Piece::Ou, _, true) => invalid_movement,
            (Piece::Ou, _, false) => full_movement,
            (Piece::Hisha, _, true) | (Piece::Kaku, _, true) => full_movement,
            (Piece::Kaku, _, false) => kaku_movement,
            (Piece::Hisha, _, false) => hisha_movement,
            (Piece::Kin, _, true) => invalid_movement,
            (Piece::Kin, true, false) => kin_movement,
            (Piece::Kin, false, false) => rev_kin_movement,
            (Piece::Gin, true, true) => kin_movement,
            (Piece::Gin, true, false) => gin_movement,
            (Piece::Gin, false, true) => rev_kin_movement,
            (Piece::Gin, false, false) => rev_gin_movement,
            (Piece::Keima, true, false) => keima_movement,
            (Piece::Keima, false, false) => rev_keima_movement,
            (Piece::Keima, true, true) => kin_movement,
            (Piece::Keima, false, true) => rev_kin_movement,
        }
    }
}

impl TryFrom<char> for Piece {
    type Error = ();

    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            'K' | 'k' => Ok(Piece::Ou),
            'R' | 'r' => Ok(Piece::Hisha),
            'B' | 'b' => Ok(Piece::Kaku),
            'G' | 'g' => Ok(Piece::Kin),
            'S' | 's' => Ok(Piece::Gin),
            'N' | 'n' => Ok(Piece::Keima),
            'L' | 'l' => Ok(Piece::Kyosha),
            'P' | 'p' => Ok(Piece::Fu),
            _ => Err(()),
        }
    }
}

struct PieceBoolPair(Piece, bool);

impl From<PieceBoolPair> for char {
    fn from(value: PieceBoolPair) -> Self {
        match value {
            PieceBoolPair(Piece::Fu, true) => ('P'),
            PieceBoolPair(Piece::Fu, false) => ('p'),
            PieceBoolPair(Piece::Ou, true) => ('K'),
            PieceBoolPair(Piece::Ou, false) => ('k'),
            PieceBoolPair(Piece::Kaku, true) => ('B'),
            PieceBoolPair(Piece::Kaku, false) => ('b'),
            PieceBoolPair(Piece::Hisha, true) => ('R'),
            PieceBoolPair(Piece::Hisha, false) => ('r'),
            PieceBoolPair(Piece::Kin, true) => ('G'),
            PieceBoolPair(Piece::Kin, false) => ('g'),
            PieceBoolPair(Piece::Gin, true) => ('S'),
            PieceBoolPair(Piece::Gin, false) => ('s'),
            PieceBoolPair(Piece::Keima, true) => ('N'),
            PieceBoolPair(Piece::Keima, false) => ('n'),
            PieceBoolPair(Piece::Kyosha, true) => ('L'),
            PieceBoolPair(Piece::Kyosha, false) => ('l'),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Hand {
    Movement {
        x: usize,
        y: usize,
        dx: isize,
        dy: isize,
        with_promote: bool,
    },
    Putting {
        piece: Piece,
        x: usize,
        y: usize,
    },
}

impl TryFrom<&str> for Hand {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let cc = value.chars().collect::<Vec<_>>();
        if cc[0].is_digit(10) {
            let x = cc[0] as usize - '0' as usize;
            let y = cc[1] as usize - 'a' as usize + 1;
            let ax = cc[2] as usize - '0' as usize;
            let ay = cc[3] as usize - 'a' as usize + 1;
            let with_promote = cc.len() == 5 && cc[4] == '+';

            Ok(Hand::Movement {
                x,
                y,
                dx: ax as isize - x as isize,
                dy: ay as isize - y as isize,
                with_promote,
            })
        } else if cc.len() != 4 || !cc[0].is_ascii_alphabetic() || cc[1] != '*' {
            Err(())
        } else {
            let piece = cc[0].try_into()?;
            let x = cc[2] as usize - '0' as usize;
            let y = cc[3] as usize - 'a' as usize + 1;
            Ok(Hand::Putting { piece, x, y })
        }
    }
}

impl From<Hand> for String {
    fn from(h: Hand) -> Self {
        let mut mv = String::new();
        match h {
            Hand::Movement {
                x,
                y,
                dx,
                dy,
                with_promote,
            } => {
                mv.push((x + '0' as usize) as u8 as char);
                mv.push((y - 1 + 'a' as usize) as u8 as char);
                mv.push((x as isize + '0' as isize + dx) as u8 as char);
                mv.push((y as isize - 1 + 'a' as isize + dy) as u8 as char);
                if with_promote {
                    mv.push('+');
                }
            }
            Hand::Putting { piece, x, y } => {
                let ch = PieceBoolPair(piece, true).into();
                mv.push(ch);
                mv.push('*');
                mv.push((x + '0' as usize) as u8 as char);
                mv.push((y - 1 + 'a' as usize) as u8 as char);
            }
        }
        mv
    }
}

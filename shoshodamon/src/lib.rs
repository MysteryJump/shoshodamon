#![allow(clippy::needless_range_loop)]
#![allow(clippy::result_unit_err)]
#![feature(bindings_after_at)]

pub mod evaluator;

use std::{
    cmp::Ordering,
    collections::{hash_map::Entry, HashMap},
    convert::{TryFrom, TryInto},
};
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

#[derive(Debug, Clone)]
pub struct Ban {
    turn: bool,
    states: [[Option<OnBoardPiece>; 9]; 9],
    primary_pieces: Vec<Piece>,
    secondary_pieces: Vec<Piece>,
}

impl Default for Ban {
    fn default() -> Self {
        Self::new()
    }
}

impl Ban {
    pub fn new() -> Self {
        Self {
            turn: true,
            primary_pieces: Vec::with_capacity(38),
            secondary_pieces: Vec::with_capacity(38),
            states: Self::initialize_states(),
        }
    }

    const fn initialize_states() -> [[Option<OnBoardPiece>; 9]; 9] {
        let mut states = [[None; 9]; 9];
        let mut i = 0;
        while i < 9 {
            states[i][7] = Some(OnBoardPiece {
                piece: Piece::Fu,
                promoted: false,
                turn: true,
            });
            states[i][6] = Some(OnBoardPiece {
                piece: Piece::Fu,
                promoted: false,
                turn: false,
            });
            i += 1;
        }

        states[0][0] = Some(OnBoardPiece {
            piece: Piece::Kyosha,
            promoted: false,
            turn: false,
        });
        states[8][0] = Some(OnBoardPiece {
            piece: Piece::Kyosha,
            promoted: false,
            turn: false,
        });
        states[8][8] = Some(OnBoardPiece {
            piece: Piece::Kyosha,
            promoted: false,
            turn: true,
        });
        states[0][8] = Some(OnBoardPiece {
            piece: Piece::Kyosha,
            promoted: false,
            turn: true,
        });

        states[1][0] = Some(OnBoardPiece {
            piece: Piece::Keima,
            promoted: false,
            turn: false,
        });
        states[7][0] = Some(OnBoardPiece {
            piece: Piece::Keima,
            promoted: false,
            turn: false,
        });
        states[1][8] = Some(OnBoardPiece {
            piece: Piece::Keima,
            promoted: false,
            turn: true,
        });
        states[7][8] = Some(OnBoardPiece {
            piece: Piece::Keima,
            promoted: false,
            turn: true,
        });

        states[2][0] = Some(OnBoardPiece {
            piece: Piece::Gin,
            promoted: false,
            turn: false,
        });
        states[6][0] = Some(OnBoardPiece {
            piece: Piece::Gin,
            promoted: false,
            turn: false,
        });
        states[2][8] = Some(OnBoardPiece {
            piece: Piece::Gin,
            promoted: false,
            turn: true,
        });
        states[6][8] = Some(OnBoardPiece {
            piece: Piece::Gin,
            promoted: false,
            turn: true,
        });

        states[3][0] = Some(OnBoardPiece {
            piece: Piece::Kin,
            promoted: false,
            turn: false,
        });
        states[5][0] = Some(OnBoardPiece {
            piece: Piece::Kin,
            promoted: false,
            turn: false,
        });
        states[3][8] = Some(OnBoardPiece {
            piece: Piece::Kin,
            promoted: false,
            turn: true,
        });
        states[5][8] = Some(OnBoardPiece {
            piece: Piece::Kin,
            promoted: false,
            turn: true,
        });

        states[4][0] = Some(OnBoardPiece {
            piece: Piece::Ou,
            promoted: false,
            turn: false,
        });
        states[4][8] = Some(OnBoardPiece {
            piece: Piece::Ou,
            promoted: false,
            turn: true,
        });

        states[1][1] = Some(OnBoardPiece {
            piece: Piece::Kaku,
            promoted: false,
            turn: false,
        });
        states[7][7] = Some(OnBoardPiece {
            piece: Piece::Kaku,
            promoted: false,
            turn: true,
        });

        states[7][1] = Some(OnBoardPiece {
            piece: Piece::Hisha,
            promoted: false,
            turn: false,
        });
        states[1][7] = Some(OnBoardPiece {
            piece: Piece::Hisha,
            promoted: false,
            turn: true,
        });
        states
    }

    pub fn move_piece(
        &mut self,
        x: usize,
        y: usize,
        dx: isize,
        dy: isize,
        with_promote: bool,
    ) -> Result<(), ()> {
        let x = 8 - x;
        let dx = -dx;
        let piece = if let Some(piece) = self.states[x][y] {
            if piece.turn == self.turn {
                piece
            } else {
                return Err(());
            }
        } else {
            return Err(());
        };

        let movements = piece
            .piece
            .get_near_piece_movement(self.turn, piece.promoted);
        if movements.contains(&(dx, dy))
            || match piece.piece {
                Piece::Kaku => self.check_kaku_movement(x, y, dx, dy),
                Piece::Hisha => self.check_hisha_movement(x, y, dx, dy),
                Piece::Kyosha => self.check_kyousha_movement(&piece, x, y, dy),
                _ => false,
            }
        {
            let xx = x as isize + dx;
            let yy = y as isize + dy;
            if xx < 0 || yy < 0 || xx >= 9 || yy >= 9 {
                return Err(());
            }
            let before = self.states[xx as usize][yy as usize];
            if let Some(before) = before {
                if before.turn == self.turn {
                    return Err(());
                }
                if piece.turn {
                    self.primary_pieces.push(before.piece);
                } else {
                    self.secondary_pieces.push(before.piece);
                }
            }
            if with_promote && (self.turn && yy > 2 || !self.turn && yy < 6) {
                return Err(());
            }
            self.states[xx as usize][yy as usize] = Some(OnBoardPiece {
                piece: piece.piece,
                promoted: with_promote || piece.promoted,
                turn: self.turn,
            });
            self.states[x][y] = None;
        }

        self.turn = !self.turn;
        Ok(())
    }

    fn check_kyousha_movement(
        &self,
        on_board_piece: &OnBoardPiece,
        x: usize,
        y: usize,
        dy: isize,
    ) -> bool {
        if on_board_piece.promoted {
            if on_board_piece.turn {
                let mut i = -1;
                loop {
                    let current = self.states[x][(y as isize + i) as usize];
                    if dy == i {
                        break;
                    }
                    if current.is_some() {
                        return false;
                    }
                    i -= 1;
                    if (y as isize + i) < 0 {
                        return false;
                    }
                }
            } else {
                let mut i = 1;
                loop {
                    let current = self.states[x][(y as isize + i) as usize];
                    if dy == i {
                        break;
                    }
                    if current.is_some() {
                        return false;
                    }
                    i += 1;
                    if (y as isize + i) > 8 {
                        return false;
                    }
                }
            }
            true
        } else {
            false
        }
    }

    fn check_hisha_movement(&self, x: usize, y: usize, dx: isize, dy: isize) -> bool {
        match (dx.cmp(&0), dy.cmp(&0)) {
            (Ordering::Equal, Ordering::Less) => {
                let mut i = -1;
                loop {
                    let current = self.states[x][(y as isize + i) as usize];
                    if dy == i {
                        break;
                    }
                    if current.is_some() {
                        return false;
                    }
                    i -= 1;
                    if (y as isize + i) < 0 {
                        return false;
                    }
                }
                true
            }
            (Ordering::Equal, Ordering::Greater) => {
                let mut i = 1;
                loop {
                    let current = self.states[x][(y as isize + i) as usize];
                    if dy == i {
                        break;
                    }
                    if current.is_some() {
                        return false;
                    }
                    i += 1;
                    if (y as isize + i) > 8 {
                        return false;
                    }
                }
                true
            }
            (Ordering::Less, Ordering::Equal) => {
                let mut i = -1;
                loop {
                    let current = self.states[(x as isize + i) as usize][y];
                    if dx == i {
                        break;
                    }
                    if current.is_some() {
                        return false;
                    }
                    i -= 1;
                    if (x as isize + i) < 0 {
                        return false;
                    }
                }
                true
            }
            (Ordering::Greater, Ordering::Equal) => {
                let mut i = 1;
                loop {
                    let current = self.states[(x as isize + i) as usize][y];
                    if dx == i {
                        break;
                    }
                    if current.is_some() {
                        return false;
                    }
                    i += 1;
                    if (x as isize + i) > 8 {
                        return false;
                    }
                }
                true
            }
            _ => false,
        }
    }

    fn check_kaku_movement(&self, x: usize, y: usize, dx: isize, dy: isize) -> bool {
        match (dx.cmp(&0), dy.cmp(&0)) {
            (Ordering::Less, Ordering::Less) => {
                let mut i = -1;
                let mut j = -1;
                loop {
                    let current = self.states[(x as isize + i) as usize][(y as isize + j) as usize];
                    if dx == i && dy == j {
                        break;
                    }
                    if current.is_some() {
                        return false;
                    }
                    i += -1;
                    j += -1;
                    if x as isize + i < 0 || y as isize + i < 0 {
                        return false;
                    }
                }
                true
            }
            (Ordering::Less, Ordering::Greater) => {
                let mut i = -1;
                let mut j = 1;
                loop {
                    let current = self.states[(x as isize + i) as usize][(y as isize + j) as usize];
                    if dx == i && dy == j {
                        break;
                    }
                    if current.is_some() {
                        return false;
                    }
                    i += -1;
                    j += 1;
                    if x as isize + i < 0 || y as isize + i > 8 {
                        return false;
                    }
                }
                true
            }
            (Ordering::Greater, Ordering::Less) => {
                let mut i = 1;
                let mut j = -1;
                loop {
                    let current = self.states[(x as isize + i) as usize][(y as isize + j) as usize];
                    if dx == i && dy == j {
                        break;
                    }
                    if current.is_some() {
                        return false;
                    }
                    i += 1;
                    j += -1;
                    if x as isize + i > 8 || y as isize + i < 0 {
                        return false;
                    }
                }
                true
            }
            (Ordering::Greater, Ordering::Greater) => {
                let mut i = 1;
                let mut j = 1;
                loop {
                    let current = self.states[(x as isize + i) as usize][(y as isize + j) as usize];
                    if dx == i && dy == j {
                        break;
                    }
                    if current.is_some() {
                        return false;
                    }
                    i += 1;
                    j += 1;
                    if x as isize + i > 8 || y as isize + i > 8 {
                        return false;
                    }
                }
                true
            }
            _ => false,
        }
    }

    pub fn put_piece(&mut self, piece: Piece, x: usize, y: usize) -> Result<(), ()> {
        let x = 8 - x;
        if self.states[x][y].is_some() {
            Err(())
        } else {
            let pos = if self.turn {
                &self.primary_pieces
            } else {
                &self.secondary_pieces
            }
            .iter()
            .position(|x| x == &piece)
            .ok_or(())?;

            match piece {
                p @ (Piece::Fu | Piece::Kyosha) => {
                    if self.turn && y == 0 || !self.turn && y == 8 {
                        Err(())
                    } else if p == Piece::Fu {
                        let diff = if self.turn { -1 } else { 1 };
                        if let Some(s) = self.states[x][(y as isize + diff) as usize] {
                            if s.piece == Piece::Ou {
                                // TODO: uchihuzume
                            }
                        }
                        for i in 0..9 {
                            if let Some(s) = &self.states[x][i] {
                                if !s.promoted && s.piece == Piece::Fu && self.turn == s.turn {
                                    return Err(());
                                }
                            }
                        }
                        Ok(())
                    } else {
                        Ok(())
                    }
                }
                Piece::Kaku | Piece::Hisha | Piece::Kin | Piece::Gin => Ok(()),
                Piece::Keima => {
                    if self.turn && y <= 1 || !self.turn && y >= 7 {
                        Err(())
                    } else {
                        Ok(())
                    }
                }
                Piece::Ou => Err(()),
            }?;
            self.states[x][y] = Some(OnBoardPiece {
                piece,
                promoted: false,
                turn: self.turn,
            });
            if self.turn {
                self.primary_pieces.remove(pos);
            } else {
                self.secondary_pieces.remove(pos);
            }
            self.turn = !self.turn;
            Ok(())
        }
    }

    pub fn is_check(&self, turn: bool) -> bool {
        let mut piece = None;
        for x in 0..9 {
            for y in 0..9 {
                if let Some(p) = &self.states[x][y] {
                    if p.piece == Piece::Ou && p.turn == turn {
                        piece = Some(((x, y), p));
                    }
                }
            }
        }

        let ((x, y), _) = piece.unwrap();
        let possibilities_near = [
            (1, 0),
            (1, 1),
            (0, 1),
            (-1, 1),
            (-1, 0),
            (-1, -1),
            (0, -1),
            (1, -1),
            (1, -2),
            (-1, -2),
            (1, 2),
            (-1, 2),
        ];
        for (dx, dy) in possibilities_near {
            let xx = dx + x as isize;
            let yy = dy + y as isize;
            if xx < 0 || yy < 0 || xx > 8 || yy > 8 {
                continue;
            }
            match &self.states[xx as usize][yy as usize] {
                Some(s) if s.turn != turn => {
                    let nears = s.piece.get_near_piece_movement(s.turn, s.promoted);
                    // TODO: needs evaluation
                    if nears.contains(&(-dx, -dy)) {
                        return true;
                    }
                }
                _ => continue,
            };
        }

        let mut yy1_ck = false;
        let mut yy2_ck = false;

        for i in 0..8 {
            let yy1 = i + y as isize;
            let yy2 = -i + y as isize;

            if !yy1_ck && (0..=8).contains(&yy1) {
                if let Some(s) = &self.states[x][yy1 as usize] {
                    if (!turn && s.piece == Piece::Kyosha || s.piece == Piece::Hisha)
                        && s.turn != turn
                    {
                        return true;
                    } else {
                        yy1_ck = true;
                    }
                }
            }

            if !yy2_ck && (0..=8).contains(&yy2) {
                if let Some(s) = &self.states[x][yy2 as usize] {
                    if (turn && s.piece == Piece::Kyosha || s.piece == Piece::Hisha)
                        && s.turn != turn
                    {
                        return true;
                    } else {
                        yy2_ck = true;
                    }
                }
            }
        }

        let mut xx1_ck = false;
        let mut xx2_ck = false;

        for i in 1..8 {
            let xx1 = i + x as isize;
            let xx2 = -i + x as isize;

            if (0..=8).contains(&xx1) && !xx1_ck {
                if let Some(s) = &self.states[xx1 as usize][y] {
                    if turn != s.turn && s.piece == Piece::Hisha {
                        return true;
                    } else {
                        xx1_ck = true;
                    }
                }
            }

            if (0..=8).contains(&xx2) && !xx2_ck {
                if let Some(s) = &self.states[xx2 as usize][y] {
                    if turn != s.turn && s.piece == Piece::Hisha {
                        return true;
                    } else {
                        xx2_ck = true;
                    }
                }
            }
        }

        let mut x1y1_ck = false;
        let mut x2y1_ck = false;
        let mut x1y2_ck = false;
        let mut x2y2_ck = false;

        for i in 0..8 {
            let xx1 = i + x as isize;
            let xx2 = -i + x as isize;
            let yy1 = i + y as isize;
            let yy2 = -i + y as isize;

            if !x1y1_ck && (0..=8).contains(&xx1) && (0..=8).contains(&yy1) {
                if let Some(s) = &self.states[xx1 as usize][yy1 as usize] {
                    if turn != s.turn && s.piece == Piece::Kaku {
                        return true;
                    } else {
                        x1y1_ck = true;
                    }
                }
            }
            if !x2y1_ck && (0..=8).contains(&xx2) && (0..=8).contains(&yy1) {
                if let Some(s) = &self.states[xx2 as usize][yy1 as usize] {
                    if turn != s.turn && s.piece == Piece::Kaku {
                        return true;
                    } else {
                        x2y1_ck = true;
                    }
                }
            }
            if !x1y2_ck && (0..=8).contains(&xx1) && (0..=8).contains(&yy2) {
                if let Some(s) = &self.states[xx1 as usize][yy2 as usize] {
                    if turn != s.turn && s.piece == Piece::Kaku {
                        return true;
                    } else {
                        x1y2_ck = true;
                    }
                }
            }
            if !x2y2_ck && (0..=8).contains(&xx2) && (0..=8).contains(&yy2) {
                if let Some(s) = &self.states[xx2 as usize][yy2 as usize] {
                    if turn != s.turn && s.piece == Piece::Kaku {
                        return true;
                    } else {
                        x2y2_ck = true;
                    }
                }
            }
        }

        false
    }

    pub fn is_check_mate(&self, turn: bool) -> bool {
        let possibilities = self.get_possibilities_ban(turn);
        possibilities.iter().all(|(x, _)| x.is_check(turn))
    }

    pub fn get_possibilities_ban(&self, turn: bool) -> Vec<(Ban, Hand)> {
        let mut bans = Vec::new();
        for x in 0..9 {
            for y in 0..9 {
                if let Some(s) = &self.states[x][y] {
                    if s.turn == turn {
                        // TODO: Kaku and Kyousha movement
                        let moves = s.piece.get_near_piece_movement(turn, s.promoted);
                        for (_, _, dx, dy) in moves
                            .iter()
                            .filter(|(dx, dy)| *dx != 0 || *dy != 0)
                            .map(|(dx, dy)| (*dx + x as isize, *dy + y as isize, dx, dy))
                            .filter(|(x, y, _, _)| *x <= 8 && *x >= 0 && *y <= 8 && *y >= 0)
                        {
                            let mut ban = (*self).clone();
                            if !s.promoted {
                                let mut ban = ban.clone();
                                if ban.move_piece(8 - x, y, -*dx, *dy, true).is_ok()
                                    && !ban.is_check(turn)
                                {
                                    bans.push((
                                        ban,
                                        Hand::Movement {
                                            x,
                                            y,
                                            dx: *dx,
                                            dy: *dy,
                                            with_promote: true,
                                        },
                                    ));
                                }
                            }
                            if ban.move_piece(8 - x, y, -*dx, *dy, false).is_ok()
                                && !ban.is_check(turn)
                            {
                                bans.push((
                                    ban,
                                    Hand::Movement {
                                        x,
                                        y,
                                        dx: *dx,
                                        dy: *dy,
                                        with_promote: false,
                                    },
                                ));
                            }
                        }
                        /*
                            if s.piece == Piece::Hisha {
                                let mut xx1_ck = false;
                                let mut xx2_ck = false;
                                let mut yy1_ck = false;
                                let mut yy2_ck = false;
                                for i in 1..=8 {
                                    let xx1 = x as isize + i;
                                    let xx2 = x as isize - i;
                                    let yy1 = y as isize + i;
                                    let yy2 = y as isize - i;
                                    if !xx1_ck && (0..=8).contains(&xx1) {
                                        let mut ban = (*self).clone();
                                        if self.states[xx1 as usize][y].is_some() {
                                            xx1_ck = true;
                                        }
                                        if ban.move_piece(x, y, -i, 0, false).is_ok()
                                            && !ban.is_check(turn)
                                        {
                                            bans.push((
                                                ban,
                                                Hand::Movement {
                                                    x,
                                                    y,
                                                    dx: i,
                                                    dy: 0,
                                                    with_promote: false,
                                                },
                                            ));
                                        }
                                        let mut ban = (*self).clone();
                                        if !s.promoted
                                            && ban.move_piece(x, y, -i, 0, true).is_ok()
                                            && !ban.is_check(turn)
                                        {
                                            bans.push((
                                                ban,
                                                Hand::Movement {
                                                    x,
                                                    y,
                                                    dx: i,
                                                    dy: 0,
                                                    with_promote: true,
                                                },
                                            ));
                                        }
                                    }

                                    if !xx2_ck && (0..=8).contains(&xx2) {
                                        let mut ban = (*self).clone();
                                        if self.states[xx2 as usize][y].is_some() {
                                            xx2_ck = true;
                                        }
                                        if ban.move_piece(x, y, i, 0, false).is_ok()
                                            && !ban.is_check(turn)
                                        {
                                            bans.push((
                                                ban,
                                                Hand::Movement {
                                                    x,
                                                    y,
                                                    dx: -i,
                                                    dy: 0,
                                                    with_promote: false,
                                                },
                                            ));
                                        }
                                        let mut ban = (*self).clone();
                                        if !s.promoted
                                            && ban.move_piece(x, y, i, 0, true).is_ok()
                                            && !ban.is_check(turn)
                                        {
                                            bans.push((
                                                ban,
                                                Hand::Movement {
                                                    x,
                                                    y,
                                                    dx: -i,
                                                    dy: 0,
                                                    with_promote: true,
                                                },
                                            ));
                                        }
                                    }

                                    if !yy1_ck && (0..=8).contains(&yy1) {
                                        let mut ban = (*self).clone();
                                        if self.states[x][yy1 as usize].is_some() {
                                            yy1_ck = true;
                                        }
                                        if ban.move_piece(x, y, 0, -i, false).is_ok()
                                            && !ban.is_check(turn)
                                        {
                                            bans.push((
                                                ban,
                                                Hand::Movement {
                                                    x,
                                                    y,
                                                    dx: 0,
                                                    dy: i,
                                                    with_promote: false,
                                                },
                                            ));
                                        }
                                        let mut ban = (*self).clone();
                                        if !s.promoted
                                            && ban.move_piece(x, y, 0, -i, true).is_ok()
                                            && !ban.is_check(turn)
                                        {
                                            bans.push((
                                                ban,
                                                Hand::Movement {
                                                    x,
                                                    y,
                                                    dx: 0,
                                                    dy: i,
                                                    with_promote: true,
                                                },
                                            ));
                                        }
                                    }

                                    if !yy2_ck && (0..=8).contains(&yy2) {
                                        if self.states[x][yy2 as usize].is_some() {
                                            yy2_ck = true;
                                        }
                                        let mut ban = (*self).clone();
                                        if ban.move_piece(x, y, 0, i, false).is_ok()
                                            && !ban.is_check(turn)
                                        {
                                            bans.push((
                                                ban,
                                                Hand::Movement {
                                                    x,
                                                    y,
                                                    dx: 0,
                                                    dy: -i,
                                                    with_promote: false,
                                                },
                                            ));
                                        }
                                        let mut ban = (*self).clone();
                                        if !s.promoted
                                            && ban.move_piece(x, y, 0, i, true).is_ok()
                                            && !ban.is_check(turn)
                                        {
                                            bans.push((
                                                ban,
                                                Hand::Movement {
                                                    x,
                                                    y,
                                                    dx: 0,
                                                    dy: -i,
                                                    with_promote: true,
                                                },
                                            ));
                                        }
                                    }
                                }
                            }
                            if s.piece == Piece::Kaku {
                                let mut x1y1_ck = false;
                                let mut x2y1_ck = false;
                                let mut x1y2_ck = false;
                                let mut x2y2_ck = false;
                                for i in 1..=8 {
                                    let xx1 = x as isize + i;
                                    let xx2 = x as isize - i;
                                    let yy1 = y as isize + i;
                                    let yy2 = y as isize - i;
                                    let xx1_contains = (0..=8).contains(&xx1);
                                    let xx2_contains = (0..=8).contains(&xx2);
                                    let yy1_contains = (0..=8).contains(&yy1);
                                    let yy2_contains = (0..=8).contains(&yy2);

                                    if !x1y1_ck && xx1_contains && yy1_contains {
                                        let mut ban = (*self).clone();
                                        if self.states[xx1 as usize][yy1 as usize].is_some() {
                                            x1y1_ck = true;
                                        }
                                        if ban.move_piece(x, y, -i, i, false).is_ok()
                                            && !ban.is_check(turn)
                                        {
                                            bans.push((
                                                ban,
                                                Hand::Movement {
                                                    x,
                                                    y,
                                                    dx: i,
                                                    dy: i,
                                                    with_promote: false,
                                                },
                                            ));
                                        }
                                        let mut ban = (*self).clone();
                                        if !s.promoted
                                            && ban.move_piece(x, y, -i, i, true).is_ok()
                                            && !ban.is_check(turn)
                                        {
                                            bans.push((
                                                ban,
                                                Hand::Movement {
                                                    x,
                                                    y,
                                                    dx: i,
                                                    dy: i,
                                                    with_promote: true,
                                                },
                                            ));
                                        }
                                    }

                                    if !x2y1_ck && xx2_contains && yy1_contains {
                                        let mut ban = (*self).clone();
                                        if self.states[xx2 as usize][yy1 as usize].is_some() {
                                            x2y1_ck = true;
                                        }
                                        if ban.move_piece(x, y, i, i, false).is_ok()
                                            && !ban.is_check(turn)
                                        {
                                            bans.push((
                                                ban,
                                                Hand::Movement {
                                                    x,
                                                    y,
                                                    dx: -i,
                                                    dy: i,
                                                    with_promote: false,
                                                },
                                            ));
                                        }
                                        let mut ban = (*self).clone();
                                        if !s.promoted
                                            && ban.move_piece(x, y, i, i, true).is_ok()
                                            && !ban.is_check(turn)
                                        {
                                            bans.push((
                                                ban,
                                                Hand::Movement {
                                                    x,
                                                    y,
                                                    dx: -i,
                                                    dy: i,
                                                    with_promote: true,
                                                },
                                            ));
                                        }
                                    }

                                    if !x1y2_ck && xx1_contains && yy2_contains {
                                        let mut ban = (*self).clone();
                                        if self.states[xx1 as usize][yy2 as usize].is_some() {
                                            x1y2_ck = true;
                                        }
                                        if ban.move_piece(x, y, -i, -i, false).is_ok()
                                            && !ban.is_check(turn)
                                        {
                                            bans.push((
                                                ban,
                                                Hand::Movement {
                                                    x,
                                                    y,
                                                    dx: i,
                                                    dy: -i,
                                                    with_promote: false,
                                                },
                                            ));
                                        }
                                        let mut ban = (*self).clone();
                                        if !s.promoted
                                            && ban.move_piece(x, y, -i, -i, true).is_ok()
                                            && !ban.is_check(turn)
                                        {
                                            bans.push((
                                                ban,
                                                Hand::Movement {
                                                    x,
                                                    y,
                                                    dx: i,
                                                    dy: -i,
                                                    with_promote: true,
                                                },
                                            ));
                                        }
                                    }

                                    if !x2y2_ck && xx2_contains && yy2_contains {
                                        if self.states[xx2 as usize][yy2 as usize].is_some() {
                                            x2y2_ck = true;
                                        }
                                        let mut ban = (*self).clone();
                                        if ban.move_piece(x, y, i, -i, false).is_ok()
                                            && !ban.is_check(turn)
                                        {
                                            bans.push((
                                                ban,
                                                Hand::Movement {
                                                    x,
                                                    y,
                                                    dx: -i,
                                                    dy: -i,
                                                    with_promote: false,
                                                },
                                            ));
                                        }
                                        let mut ban = (*self).clone();
                                        if !s.promoted
                                            && ban.move_piece(x, y, i, -i, true).is_ok()
                                            && !ban.is_check(turn)
                                        {
                                            bans.push((
                                                ban,
                                                Hand::Movement {
                                                    x,
                                                    y,
                                                    dx: -i,
                                                    dy: -i,
                                                    with_promote: true,
                                                },
                                            ));
                                        }
                                    }
                                }
                            }
                            if s.piece == Piece::Kyosha && !s.promoted {
                                for i in 0..8 {
                                    let yy = if s.turn {
                                        y as isize - i
                                    } else {
                                        y as isize + i
                                    };
                                    if (0..=8).contains(&yy) {
                                        let mut ban = (*self).clone();
                                        if ban
                                            .move_piece(x, y, 0, if s.turn { -i } else { i }, false)
                                            .is_ok()
                                            && !ban.is_check(s.turn)
                                        {
                                            bans.push((
                                                ban,
                                                Hand::Movement {
                                                    x,
                                                    y,
                                                    dx: 0,
                                                    dy: if s.turn { -i } else { i },
                                                    with_promote: true,
                                                },
                                            ));
                                        }
                                        let mut ban = (*self).clone();
                                        if ban
                                            .move_piece(x, y, 0, if s.turn { -i } else { i }, true)
                                            .is_ok()
                                            && !ban.is_check(s.turn)
                                        {
                                            bans.push((
                                                ban,
                                                Hand::Movement {
                                                    x,
                                                    y,
                                                    dx: 0,
                                                    dy: if s.turn { -i } else { i },
                                                    with_promote: true,
                                                },
                                            ));
                                        }
                                    }
                                }
                            }

                        */
                    }
                } else {
                    let pieces = if turn {
                        &self.primary_pieces
                    } else {
                        &self.secondary_pieces
                    };
                    for piece in pieces {
                        let mut ban = (*self).clone();
                        if ban.put_piece(*piece, x, y).is_ok() && !ban.is_check(*ban.turn()) {
                            bans.push((
                                ban,
                                Hand::Putting {
                                    piece: *piece,
                                    x,
                                    y,
                                },
                            ));
                        }
                    }
                }
            }
        }

        bans
    }

    pub fn parse_sfen(sfen: &str) -> Result<Self, ()> {
        let splited = sfen.trim().split_ascii_whitespace().collect::<Vec<_>>();
        let (bans, hand, havings) = (
            splited[0].trim().split('/').collect::<Vec<_>>(),
            splited[1],
            splited[2],
        );
        let mut states: [[Option<OnBoardPiece>; 9]; 9] = [[None; 9]; 9];
        for i in 0..9 {
            let mut current_ind = 0;
            let mut chars = bans[i].chars();
            while let Some(ch) = chars.next() {
                if ch.is_digit(10) {
                    let num = ch as i32 - '0' as i32;
                    current_ind += num;
                } else if ch.is_ascii_alphabetic() {
                    let is_promoted = ch == '+';
                    let piece = if is_promoted {
                        chars.next().ok_or(())?
                    } else {
                        ch
                    }
                    .try_into()?;

                    states[current_ind as usize][i] = Some(OnBoardPiece {
                        piece,
                        promoted: is_promoted,
                        turn: ch.is_uppercase(),
                    });
                    current_ind += 1;
                } else {
                    return Err(());
                }
            }
        }

        let turn = match hand {
            "b" => true,
            "w" => false,
            _ => return Err(()),
        };

        let mut primary_havings: Vec<Piece> = Vec::with_capacity(38);
        let mut secondary_havings = Vec::with_capacity(38);
        let mut havings_chars = havings.chars();

        if havings != "-" {
            while let Some(ch) = havings_chars.next() {
                let (ch, num) = if ch.is_digit(10) {
                    let num = ch as i32 - '0' as i32;
                    (havings_chars.next().ok_or(())?, num)
                } else {
                    (ch, 1)
                };
                if ch.is_ascii_alphabetic() {
                    let p = ch.try_into()?;
                    if ch.is_uppercase() {
                        (0..num).for_each(|_| primary_havings.push(p));
                    } else {
                        (0..num).for_each(|_| secondary_havings.push(p));
                    }
                } else {
                    return Err(());
                }
            }
        }

        Ok(Ban {
            turn,
            states,
            primary_pieces: primary_havings,
            secondary_pieces: secondary_havings,
        })
    }

    pub fn to_sfen(&self) -> String {
        let mut sfen = String::new();
        let mut reverse = [[None; 9]; 9];
        for i in 0..9 {
            for j in 0..9 {
                reverse[i][j] = self.states[j][i];
            }
        }
        for y_row in &reverse {
            let mut none_count = 0;
            for x_c in y_row {
                if let Some(piece) = x_c {
                    if none_count != 0 {
                        sfen.push_str(&none_count.to_string());
                        none_count = 0;
                    }
                    let s: String = <&OnBoardPiece>::into(piece);
                    sfen.push_str(&s);
                } else {
                    none_count += 1;
                }
            }
            if none_count != 0 {
                sfen.push_str(&none_count.to_string());
            }
            sfen.push('/');
        }
        sfen.pop();
        sfen.push(' ');

        sfen.push(if self.turn { 'b' } else { 'w' });
        sfen.push(' ');

        if !self.primary_pieces.is_empty() || !self.secondary_pieces.is_empty() {
            let mut map = HashMap::new();
            for piece in self
                .primary_pieces
                .iter()
                .map(|p| PieceBoolPair(*p, true))
                .chain(
                    self.secondary_pieces
                        .iter()
                        .map(|p| PieceBoolPair(*p, false)),
                )
            {
                let ch: char = piece.into();
                if let Entry::Vacant(e) = map.entry(ch) {
                    e.insert(1);
                } else {
                    let c = map.get_mut(&ch).unwrap();
                    *c += 1;
                }
            }
            for (p, c) in map {
                if c != 1 {
                    sfen.push_str(&c.to_string());
                }
                sfen.push(p);
            }
        } else {
            sfen.push('-');
        }
        sfen.push(' ');

        sfen.push('1');
        sfen
    }

    /// Get a reference to the ban's turn.
    pub fn turn(&self) -> &bool {
        &self.turn
    }
}

#[derive(Debug, Clone, Copy)]
struct OnBoardPiece {
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
            let x = cc[0] as usize - '1' as usize;
            let y = cc[1] as usize - 'a' as usize;
            let ax = cc[2] as usize - '1' as usize;
            let ay = cc[3] as usize - 'a' as usize;
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
            let x = cc[2] as usize - '1' as usize;
            let y = cc[3] as usize - 'a' as usize;
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
                let x = 8 - x;
                let dx = -dx;
                mv.push((x + '1' as usize) as u8 as char);
                mv.push((y + 'a' as usize) as u8 as char);
                mv.push((x as isize + '1' as isize + dx) as u8 as char);
                mv.push((y as isize + 'a' as isize + dy) as u8 as char);
                if with_promote {
                    mv.push('+');
                }
            }
            Hand::Putting { piece, x, y } => {
                let x = 8 - x;
                let ch = PieceBoolPair(piece, true).into();
                mv.push(ch);
                mv.push('*');
                mv.push((x + '1' as usize) as u8 as char);
                mv.push((y + 'a' as usize) as u8 as char);
            }
        }
        mv
    }
}

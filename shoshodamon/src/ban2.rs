use std::{
    cmp::Ordering,
    collections::{hash_map::Entry, HashMap},
    convert::TryInto,
};

use crate::{Hand, OnBoardPiece, Piece, PieceBoolPair};

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
pub struct Ban2 {
    pub turn: bool,
    states: [Option<OnBoardPiece>; 81],
    pub primary_pieces: Vec<Piece>,
    pub secondary_pieces: Vec<Piece>,
}

impl Ban2 {
    pub fn new() -> Self {
        Self::from_sfen(super::START_POS).unwrap()
    }

    /// Create a `Ban` from given sfen
    pub fn from_sfen(sfen: &str) -> Result<Self, &'static str> {
        let splited = sfen.trim().split_ascii_whitespace().collect::<Vec<_>>();
        let (bans, hand, havings) = (
            splited[0].trim().split('/').collect::<Vec<_>>(),
            splited[1],
            splited[2],
        );

        let turn = match hand {
            "b" => true,
            "w" => false,
            _ => return Err("Cannot parse sfen - near turn"),
        };

        let mut primary_havings: Vec<Piece> = Vec::with_capacity(38);
        let mut secondary_havings = Vec::with_capacity(38);
        let mut havings_chars = havings.chars();
        if havings != "-" {
            while let Some(ch) = havings_chars.next() {
                let (ch, num) = if ch.is_digit(10) {
                    let num = ch as i32 - '0' as i32;
                    (
                        havings_chars
                            .next()
                            .ok_or("Cannot parse sfen - near havings")?,
                        num,
                    )
                } else {
                    (ch, 1)
                };
                if ch.is_ascii_alphabetic() {
                    let p = ch
                        .try_into()
                        .map_err(|_| "Cannot parse sfen - near havings")?;
                    if ch.is_uppercase() {
                        (0..num).for_each(|_| primary_havings.push(p));
                    } else {
                        (0..num).for_each(|_| secondary_havings.push(p));
                    }
                } else {
                    return Err("Cannot parse sfen - near havings");
                }
            }
        }

        let mut ban2 = Ban2 {
            turn,
            states: [None; 81],
            primary_pieces: primary_havings,
            secondary_pieces: secondary_havings,
        };

        for i in 0..9 {
            let mut current_ind = 0;
            let mut chars = bans[i].chars();
            while let Some(ch) = chars.next() {
                if ch.is_digit(10) {
                    let num = ch as usize - '0' as usize;
                    current_ind += num;
                } else if ch.is_ascii_alphabetic() || ch == '+' {
                    let is_promoted = ch == '+';
                    let piece = if is_promoted {
                        let c = chars.next().ok_or("Cannot parse board - near board1")?;
                        (
                            c.try_into()
                                .map_err(|_| "Cannot parse board - near board2")?,
                            c.is_uppercase(),
                        )
                    } else {
                        (
                            ch.try_into()
                                .map_err(|_| "Cannot parse board - near board2")?,
                            ch.is_uppercase(),
                        )
                    };

                    ban2.update_position(
                        9 - current_ind,
                        i + 1,
                        Some(OnBoardPiece {
                            piece: piece.0,
                            promoted: is_promoted,
                            turn: piece.1,
                        }),
                    );

                    current_ind += 1;
                } else {
                    return Err("Cannot parse board - near board3");
                }
            }
        }
        Ok(ban2)
    }

    /// Create a sfen-formatted string from current `Ban` status
    pub fn to_sfen(&self) -> String {
        let mut sfen = String::new();
        for y in 1..=9 {
            let mut none_count = 0;
            for x in (1..=9).rev() {
                if let Some(piece) = self.get_position(x, y) {
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

    /// Get a piece with given x and y. x and y needs 1-indexed
    #[inline]
    pub fn get_position(&self, x: usize, y: usize) -> &Option<OnBoardPiece> {
        if (1..=9).contains(&x) && (1..=9).contains(&y) {
            &self.states[(x - 1) + (y - 1) * 9]
        } else {
            eprintln!("Invalid operation in get_position: {} {}", x, y);
            &None
        }
    }

    /// Update a piece with given x and y. x and y needs 1-indexed
    #[inline]
    fn update_position(
        &mut self,
        x: usize,
        y: usize,
        on_board_piece: Option<OnBoardPiece>,
    ) -> Option<OnBoardPiece> {
        if (1..=9).contains(&x) && (1..=9).contains(&y) {
            let ret = self.states[(x - 1) + (y - 1) * 9];
            self.states[(x - 1) + (y - 1) * 9] = on_board_piece;
            ret
        } else {
            eprintln!("Invalid operation in update_position");
            None
        }
    }

    /// Move piece with error check. all of x and y need 1-indexed
    // TODO: need Hisha, Kaku and Kyousha check
    pub fn move_piece(
        &mut self,
        before_x: usize,
        before_y: usize,
        after_x: usize,
        after_y: usize,
        with_promote: bool,
    ) -> Result<(), &'static str> {
        let piece = if let Some(piece) = *self.get_position(before_x, before_y) {
            if piece.turn != self.turn {
                Err("Specified position is not your piece")
            } else {
                Ok(piece)
            }
        } else {
            Err("Does not exist any piece at given position")
        }?;

        let dx = after_x as isize - before_x as isize;
        let dy = after_y as isize - before_y as isize;

        if with_promote && (self.turn && after_y > 3 || !self.turn && after_y < 7) {
            return Err("Cannot promote your piece at specified moved position");
        } else if with_promote && piece.promoted {
            return Err("Cannot promote your piece which already promoted");
        }

        let movements = piece
            .piece
            .get_near_piece_movement(self.turn, piece.promoted);

        if movements.contains(&(dx, dy))
            || match piece.piece {
                Piece::Kaku => self.check_kaku_movement(before_x, before_y, dx, dy),
                Piece::Hisha => self.check_hisha_movement(before_x, before_y, dx, dy),
                Piece::Kyosha => self.check_kyosha_movement(before_x, before_y, dy, &piece),
                _ => false,
            }
        {
            if let Some(last_piece) = *self.get_position(after_x, after_y) {
                if last_piece.turn == self.turn {
                    return Err("Cannot move piece to your piece");
                }

                if piece.turn {
                    self.primary_pieces.push(last_piece.piece);
                } else {
                    self.secondary_pieces.push(last_piece.piece);
                }
            }
            self.update_position(
                after_x,
                after_y,
                Some(OnBoardPiece {
                    piece: piece.piece,
                    promoted: piece.promoted || with_promote,
                    turn: self.turn,
                }),
            );
            self.update_position(before_x, before_y, None);
            self.turn = !self.turn;
            Ok(())
        } else {
            Err("Cannot move piece to specified position")
        }
    }

    fn check_kaku_movement(&self, x: usize, y: usize, dx: isize, dy: isize) -> bool {
        match (dx.cmp(&0), dy.cmp(&0)) {
            (
                ox @ (Ordering::Less | Ordering::Greater),
                oy @ (Ordering::Less | Ordering::Greater),
            ) => {
                let xdiff = if ox == Ordering::Less { -1 } else { 1 };
                let ydiff = if oy == Ordering::Greater { -1 } else { 1 };
                let mut i = xdiff;
                let mut j = ydiff;
                loop {
                    if x as isize + i < 1
                        || x as isize + i > 9
                        || y as isize + j < 1
                        || y as isize + j > 9
                    {
                        return false;
                    }
                    let current =
                        self.get_position((x as isize + i) as usize, (y as isize + j) as usize);
                    if dx == i && dy == j {
                        break;
                    }
                    if current.is_some() {
                        return false;
                    }
                    i += xdiff;
                    j += ydiff;
                }
                true
            }
            _ => false,
        }
    }

    fn check_hisha_movement(&self, x: usize, y: usize, dx: isize, dy: isize) -> bool {
        match (dx.cmp(&0), dy.cmp(&0)) {
            (Ordering::Equal, o @ (Ordering::Less | Ordering::Greater)) => {
                let diff = if o == Ordering::Less { -1 } else { 1 };
                let mut i = diff;
                loop {
                    if y as isize + i < 1 || y as isize + i > 9 {
                        return false;
                    }
                    let current = self.get_position(x, (y as isize + i) as usize);
                    if dy == i {
                        break;
                    }
                    if current.is_some() {
                        return false;
                    }
                    i += diff;
                }
                true
            }
            (o @ (Ordering::Less | Ordering::Greater), Ordering::Equal) => {
                let diff = if o == Ordering::Less { -1 } else { 1 };
                let mut i = diff;
                loop {
                    if x as isize + i < 1 || x as isize + i > 9 {
                        return false;
                    }
                    let current = self.get_position((x as isize + i) as usize, y);
                    if dx == i {
                        break;
                    }
                    if current.is_some() {
                        return false;
                    }
                    i += diff;
                }
                true
            }
            _ => false,
        }
    }

    fn check_kyosha_movement(
        &self,
        x: usize,
        y: usize,
        dy: isize,
        on_board_piece: &OnBoardPiece,
    ) -> bool {
        if on_board_piece.promoted {
            let diff = if on_board_piece.turn { -1 } else { 1 };
            let mut i = diff;
            loop {
                if y as isize + i < 1 || y as isize + i > 9 {
                    return false;
                }
                let current = self.get_position(x, (y as isize + i) as usize);
                if dy == i {
                    break;
                }
                if current.is_some() {
                    return false;
                }
                i += diff;
            }
            true
        } else {
            false
        }
    }

    /// Put a piece with error check. position needs 1-indexed
    pub fn put_piece(&mut self, piece: Piece, x: usize, y: usize) -> Result<(), &'static str> {
        if self.get_position(x, y).is_some() {
            return Err("Cannot put a piece on a piece which already exists at specified position");
        }

        let pieces = if self.turn {
            &self.primary_pieces
        } else {
            &self.secondary_pieces
        };

        let pos = pieces
            .iter()
            .position(|x| x == &piece)
            .ok_or("Cannot find the piece in your havings.")?;

        match piece {
            p @ (Piece::Fu | Piece::Kyosha) => {
                if self.turn && y == 1 || !self.turn && y == 9 {
                    Err("Cannot put a piece at specified place")
                } else if p == Piece::Fu {
                    // TODO: uchifuzume

                    for i in 1..=9 {
                        if let Some(s) = self.get_position(x, i) {
                            if !s.promoted && s.piece == Piece::Fu && self.turn == s.turn {
                                return Err("This movements cause nifu");
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
                if self.turn && y < 3 || !self.turn && y > 7 {
                    Err("Cannot put Keima at specified place")
                } else {
                    Ok(())
                }
            }
            Piece::Ou => panic!(),
        }?;

        if self.turn {
            self.primary_pieces.remove(pos);
        } else {
            self.secondary_pieces.remove(pos);
        }
        self.update_position(
            x,
            y,
            Some(OnBoardPiece {
                piece,
                promoted: false,
                turn: self.turn,
            }),
        );

        self.turn = !self.turn;
        Ok(())
    }

    /// Check given turn checked.
    pub fn is_check(&self, turn: bool) -> bool {
        let mut ou = None;
        for x in 1..=9 {
            for y in 1..=9 {
                if let Some(piece) = self.get_position(x, y) {
                    if piece.piece == Piece::Ou && piece.turn == turn {
                        ou = Some(((x, y), piece));
                    }
                }
            }
        }

        let ((x, y), _) = ou.unwrap();
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
            if xx < 1 || yy < 1 || xx > 9 || yy > 9 {
                continue;
            }
            match self.get_position(xx as usize, yy as usize) {
                Some(s) if s.turn != turn => {
                    let nears = s.piece.get_near_piece_movement(s.turn, s.promoted);
                    if nears.contains(&(-dx, -dy)) {
                        return true;
                    }
                }
                _ => continue,
            };
        }

        for pn in [true, false] {
            for i in 1..=8 {
                let yy = if pn { i } else { -i } + y as isize;
                if (1..=9).contains(&yy) {
                    if let Some(s) = self.get_position(x, yy as usize) {
                        if ((turn ^ pn) && s.piece == Piece::Kyosha || s.piece == Piece::Hisha)
                            && s.turn != turn
                        {
                            return true;
                        } else {
                            break;
                        }
                    }
                }
            }
        }

        for pn in [true, false] {
            for i in 1..=8 {
                let xx = if pn { i } else { -i } + x as isize;
                if (1..=9).contains(&xx) {
                    if let Some(s) = self.get_position(xx as usize, y) {
                        if s.piece == Piece::Hisha && s.turn != turn {
                            return true;
                        } else {
                            break;
                        }
                    }
                }
            }
        }

        for (pn_x, pn_y) in [(true, true), (true, false), (false, true), (false, false)] {
            for i in 1..=8 {
                let xx = if pn_x { i } else { -i } + x as isize;
                let yy = if pn_y { i } else { -i } + y as isize;
                if (1..=9).contains(&xx) && (1..=9).contains(&yy) {
                    if let Some(s) = self.get_position(xx as usize, yy as usize) {
                        if s.piece == Piece::Kaku && s.turn != turn {
                            return true;
                        } else {
                            break;
                        }
                    }
                }
            }
        }
        false
    }

    /// Check given turn check mated.
    pub fn is_check_mate(&self, turn: bool) -> bool {
        let possibilities = self.get_possibility_bans(turn);
        possibilities.iter().all(|(x, _)| x.is_check(turn))
    }

    /// Get all possiblities of next turn
    pub fn get_possibility_bans(&self, turn: bool) -> Vec<(Ban2, Hand)> {
        let mut bans = Vec::new();
        for x in 1..=9 {
            for y in 1..=9 {
                if let Some(s) = self.get_position(x, y) {
                    if s.turn == turn {
                        let moves = s.piece.get_near_piece_movement(turn, s.promoted);
                        for (_, _, dx, dy) in moves
                            .iter()
                            .filter(|(dx, dy)| *dx != 0 || *dy != 0)
                            .map(|(dx, dy)| (*dx + x as isize, *dy + y as isize, dx, dy))
                            .filter(|(x, y, _, _)| (1..=9).contains(x) && (1..=9).contains(y))
                        {
                            let ax = (x as isize + dx) as usize;
                            let ay = (y as isize + dy) as usize;
                            let mut ban = (*self).clone();
                            if !s.promoted {
                                let mut ban = ban.clone();
                                if ban.move_piece(x, y, ax, ay, true).is_ok() && !ban.is_check(turn)
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
                            if ban.move_piece(x, y, ax, ay, false).is_ok() && !ban.is_check(turn) {
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

                        if s.piece == Piece::Hisha || (s.piece == Piece::Kyosha && !s.promoted) {
                            if s.piece == Piece::Hisha {
                                for x_pn in [true, false] {
                                    for i in 2..=8 {
                                        let diff = if x_pn { i } else { -i };
                                        let xx = x as isize + diff;
                                        if (2..=9).contains(&xx) {
                                            for with_promote in [true, false] {
                                                if s.promoted && with_promote {
                                                    continue;
                                                }
                                                let mut ban = (*self).clone();
                                                if ban
                                                    .move_piece(x, y, xx as usize, y, with_promote)
                                                    .is_ok()
                                                    && !ban.is_check(turn)
                                                {
                                                    bans.push((
                                                        ban,
                                                        Hand::Movement {
                                                            x,
                                                            y,
                                                            dx: diff,
                                                            dy: 0,
                                                            with_promote,
                                                        },
                                                    ))
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            for y_pn in [true, false] {
                                if s.piece == Piece::Kyosha && !(y_pn ^ turn) {
                                    continue;
                                }

                                for i in 2..=8 {
                                    let diff = if y_pn { i } else { -i };
                                    let yy = y as isize + diff;
                                    if (2..=9).contains(&yy) {
                                        for with_promote in [true, false] {
                                            if s.promoted && with_promote {
                                                continue;
                                            }
                                            let mut ban = (*self).clone();
                                            if ban
                                                .move_piece(x, y, x, yy as usize, with_promote)
                                                .is_ok()
                                                && !ban.is_check(turn)
                                            {
                                                bans.push((
                                                    ban,
                                                    Hand::Movement {
                                                        x,
                                                        y,
                                                        dx: 0,
                                                        dy: diff,
                                                        with_promote,
                                                    },
                                                ))
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        if s.piece == Piece::Kaku {
                            for (x_pn, y_pn) in
                                [(true, true), (true, false), (false, true), (false, false)]
                            {
                                for i in 2..=8 {
                                    let x_diff = if x_pn { i } else { -i };
                                    let y_diff = if y_pn { i } else { -i };
                                    let xx = x as isize + x_diff;
                                    let yy = y as isize + y_diff;

                                    if (2..=9).contains(&xx) && (2..=9).contains(&yy) {
                                        for with_promote in [true, false] {
                                            if s.promoted && with_promote {
                                                continue;
                                            }
                                            let mut ban = (*self).clone();
                                            if ban
                                                .move_piece(
                                                    x,
                                                    y,
                                                    xx as usize,
                                                    yy as usize,
                                                    with_promote,
                                                )
                                                .is_ok()
                                                && !ban.is_check(turn)
                                            {
                                                bans.push((
                                                    ban,
                                                    Hand::Movement {
                                                        x,
                                                        y,
                                                        dx: x_diff,
                                                        dy: y_diff,
                                                        with_promote,
                                                    },
                                                ))
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    let pieces = if turn {
                        &self.primary_pieces
                    } else {
                        &self.secondary_pieces
                    };
                    for piece in pieces {
                        let mut ban = (*self).clone();
                        if ban.put_piece(*piece, x, y).is_ok() && !ban.is_check(turn) {
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
}

impl Default for Ban2 {
    fn default() -> Self {
        Self::new()
    }
}

#[test]
fn feature() {
    let ban2 =
        Ban2::from_sfen("lns1+Bgsnl/1r1g1k1b1/pppppppp1/8p/1P7/4PR3/P1PP1PPPP/9/LNSGKGSNL w - 1")
            .unwrap();
    println!("{:?}", ban2.get_position(5, 1));
    println!("{}", ban2.to_sfen());
}

use core::*;
use bitboard::*;
use moves::*;
use board::*;
use tables::*;

use std::collections::HashMap;

bitflags! {
    pub struct CastlingRights: u8 {
        const WHITE_KINGSIDE  = 0b0001;
        const WHITE_QUEENSIDE = 0b0010;
        const BLACK_KINGSIDE  = 0b0100;
        const BLACK_QUEENSIDE = 0b1000;
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Game {
    pub board: Board,
    pub to_move: Color,
    pub ep_square: Option<Square>,
    pub castling_rights: CastlingRights,
    pub fifty_move_count: u8,
    pub moves: u16
}

impl Game {
    pub fn starting_position() -> Game {
        Game {
            board: Board::starting_position(),
            to_move: Color::White,
            ep_square: None,
            castling_rights: CastlingRights::all(),
            fifty_move_count: 0,
            moves: 1,
        }
    }

    pub fn empty_position() -> Game {
        Game {
            board: Board::empty_position(),
            to_move: Color::White,
            ep_square: None,
            castling_rights: CastlingRights::empty(),
            fifty_move_count: 0,
            moves: 1
        }
    }

    pub fn from_fen(fen: &'static str) -> Option<Game> {
        let words: Vec<&str> = fen.split(' ').collect();

        if words.len() != 6 {
            return None;
        }

        let mut game = Game::empty_position();

        use PieceType::*;
        use Color::*;

        { // build up the game board
            let mut current_square: Square = Square::new(63);

            let decrement_square = |sq: &mut Square, n: u32| {
                if sq.unwrap() > 0 {
                    *sq = Square::new(sq.unwrap() - n);
                }
            };

            let mut add_piece = |color: Color, piece: PieceType, sq: &mut Square| {
                game.board.set_piece_bit(color, piece, *sq);
                decrement_square(sq, 1);
            };

            for ch in words[0].chars() {
                match ch {
                    'p' => add_piece(Black , Pawn   , &mut current_square) ,
                    'n' => add_piece(Black , Knight , &mut current_square) ,
                    'b' => add_piece(Black , Bishop , &mut current_square) ,
                    'r' => add_piece(Black , Rook   , &mut current_square) ,
                    'q' => add_piece(Black , Queen  , &mut current_square) ,
                    'k' => add_piece(Black , King   , &mut current_square) ,
                    'P' => add_piece(White , Pawn   , &mut current_square) ,
                    'N' => add_piece(White , Knight , &mut current_square) ,
                    'B' => add_piece(White , Bishop , &mut current_square) ,
                    'R' => add_piece(White , Rook   , &mut current_square) ,
                    'Q' => add_piece(White , Queen  , &mut current_square) ,
                    'K' => add_piece(White , King   , &mut current_square) ,
                    '1' => decrement_square(&mut current_square, 1),
                    '2' => decrement_square(&mut current_square, 2),
                    '3' => decrement_square(&mut current_square, 3),
                    '4' => decrement_square(&mut current_square, 4),
                    '5' => decrement_square(&mut current_square, 5),
                    '6' => decrement_square(&mut current_square, 6),
                    '7' => decrement_square(&mut current_square, 7),
                    '8' => decrement_square(&mut current_square, 8),
                    '/' => {},
                    _ => return None
                }
            }
        }

        match words[1] {
            "w" => game.to_move = White,
            "b" => game.to_move = Black,
            _ => return None
        }

        for ch in words[2].chars() {
            match ch {
                'K' => game.castling_rights |= CastlingRights::WHITE_KINGSIDE,
                'Q' => game.castling_rights |= CastlingRights::WHITE_QUEENSIDE,
                'k' => game.castling_rights |= CastlingRights::BLACK_KINGSIDE,
                'q' => game.castling_rights |= CastlingRights::BLACK_QUEENSIDE,
                '-' => {},
                _ => return None
            }
        }

        match words[3] {
            "-" => game.ep_square = None,
            _ => match Square::from_algebraic(words[3]) {
                None => return None,
                Some(sq) => game.ep_square = Some(sq)
            }
        }

        match words[4].parse::<u8>() {
            Err(_) => return None,
            Ok(x) => game.fifty_move_count = x
        }

        match words[5].parse::<u16>() {
            Err(_) => return None,
            Ok(x) => game.moves = x
        }

        return Some(game);
    }

    pub fn make_move(&mut self, m: Move) {
        let from_sq        = m.from();
        let from_bit       = from_sq.bitrep();
        let to_sq          = m.to();
        let to_bit         = to_sq.bitrep();
        let from_to_bit    = from_bit | to_bit;
        let is_capture     = m.is_capture();
        let flag           = m.flag();
        let moving_color   = self.to_move;
        let opponent_color = !moving_color;

        use Color::*;
        use PieceType::*;

        let moved_piece = self.board.piece_at(from_sq).unwrap().ptype;
        let captured_piece =
            if flag == EP_CAPTURE_FLAG {
                match opponent_color {
                    White => self.board.piece_at(Square::new(self.ep_square.unwrap().unwrap() + 8)),
                    Black => self.board.piece_at(Square::new(self.ep_square.unwrap().unwrap() - 8))
                }
            } else {
                self.board.piece_at(to_sq)
            };

        assert!(is_capture == captured_piece.is_some());

        //TODO: add moving/captured piece type to Move structure
        match moved_piece {
            Pawn => {
                *self.board.get_pieces_mut(self.to_move, Pawn) ^= from_to_bit;
                *self.board.occupied_by_mut(self.to_move) ^= from_to_bit;

                if flag == DOUBLE_PAWN_PUSH_FLAG {
                    self.ep_square = match moving_color {
                        White => Some(Square::new(to_sq.unwrap() - 8)),
                        Black => Some(Square::new(to_sq.unwrap() + 8))
                    }
                }


                // TODO: promotions

                if is_capture {
                    if flag == EP_CAPTURE_FLAG {
                        assert!(self.ep_square.is_some());

                        let captured_bit = match moving_color {
                            White => self.ep_square.unwrap().bitrep().shifted_up(),
                            Black => self.ep_square.unwrap().bitrep().shifted_down()
                        };

                        *self.board.get_pieces_mut(opponent_color, Pawn) ^= captured_bit;
                        *self.board.occupied_by_mut(opponent_color) ^= captured_bit;
                    } else {
                        *self.board.get_pieces_mut(opponent_color, captured_piece.unwrap().ptype) ^= to_bit;
                        *self.board.occupied_by_mut(opponent_color) ^= to_bit;
                    }
                }

            },

            Knight => {
                *self.board.get_pieces_mut(self.to_move, Knight) ^= from_to_bit;
                *self.board.occupied_by_mut(self.to_move) ^= from_to_bit;
                if is_capture {
                    *self.board.get_pieces_mut(opponent_color, captured_piece.unwrap().ptype) ^= to_bit;
                    *self.board.occupied_by_mut(opponent_color) ^= to_bit;
                }
            },

            Bishop => {
                *self.board.get_pieces_mut(self.to_move, Bishop) ^= from_to_bit;
                *self.board.occupied_by_mut(self.to_move) ^= from_to_bit;
                if is_capture {
                    *self.board.get_pieces_mut(opponent_color, captured_piece.unwrap().ptype) ^= to_bit;
                    *self.board.occupied_by_mut(opponent_color) ^= to_bit;
                }
            },

            Rook => {
                *self.board.get_pieces_mut(self.to_move, Rook) ^= from_to_bit;
                *self.board.occupied_by_mut(self.to_move) ^= from_to_bit;

                match moving_color {
                    White =>
                        if from_sq.idx() == 0 {
                            self.castling_rights.remove(CastlingRights::WHITE_KINGSIDE);
                        } else if from_sq.idx() == 7 {
                            self.castling_rights.remove(CastlingRights::WHITE_QUEENSIDE);
                        },

                    Black =>
                        if from_sq.idx() == 63 {
                            self.castling_rights.remove(CastlingRights::BLACK_QUEENSIDE);
                        } else if from_sq.idx() == 56 {
                            self.castling_rights.remove(CastlingRights::BLACK_KINGSIDE);
                        }
                }

                if is_capture {
                    *self.board.get_pieces_mut(opponent_color, captured_piece.unwrap().ptype) ^= to_bit;
                    *self.board.occupied_by_mut(opponent_color) ^= to_bit;
                }
            },

            Queen => {
                *self.board.get_pieces_mut(self.to_move, Queen) ^= from_to_bit;
                *self.board.occupied_by_mut(self.to_move) ^= from_to_bit;
                if is_capture {
                    *self.board.get_pieces_mut(opponent_color, captured_piece.unwrap().ptype) ^= to_bit;
                    *self.board.occupied_by_mut(opponent_color) ^= to_bit;
                }
            },

            King => {
                *self.board.get_pieces_mut(self.to_move, King) ^= from_to_bit;
                *self.board.occupied_by_mut(self.to_move) ^= from_to_bit;

                match moving_color {
                    White => {
                            self.castling_rights.remove(CastlingRights::WHITE_KINGSIDE);
                            self.castling_rights.remove(CastlingRights::WHITE_QUEENSIDE);
                    }

                    Black => {
                            self.castling_rights.remove(CastlingRights::BLACK_QUEENSIDE);
                            self.castling_rights.remove(CastlingRights::BLACK_KINGSIDE);
                    }
                }

                if is_capture {
                    *self.board.get_pieces_mut(opponent_color, captured_piece.unwrap().ptype) ^= to_bit;
                    *self.board.occupied_by_mut(opponent_color) ^= to_bit;
                }
            },
        }

        if flag != DOUBLE_PAWN_PUSH_FLAG {
            self.ep_square = None;
        }

        if is_capture || moved_piece == Pawn {
            self.fifty_move_count = 0;
        } else {
            // self.fifty_move_count += 1;
        }

        self.to_move = !self.to_move;

    }
}

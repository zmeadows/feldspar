use core::*;
use moves::*;
use board::*;
use tables::*;

bitflags! {
    pub struct CastlingRights: u32 {
        const WHITE_KINGSIDE  = 0b0001;
        const WHITE_QUEENSIDE = 0b0010;
        const BLACK_KINGSIDE  = 0b0100;
        const BLACK_QUEENSIDE = 0b1000;
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Game {
    board: Board,
    to_move: Color,
    ep_square: Option<Square>,
    castling_rights: CastlingRights,
    fifty_move_count: usize
}


impl Game {
    pub fn new() -> Game {
        Game {
            board: Board::starting_position(),
            to_move: Color::White,
            ep_square: None,
            castling_rights: CastlingRights::all(),
            fifty_move_count: 0
        }
    }

    pub fn new_from_fen(fen: String) -> Option<Game> {
        let words: Vec<&str> = fen.split(' ').collect();

        if words.len() != 6 {
            return None;
        }

        let mut game = Game::new();
        game.board = Board::starting_position();
        let mut current_square: Square = Square::new(63);

        for ch in words[0].chars() {
            match ch {
                _ => return None
            }
        }

        return Some(game);
    }

    pub fn fill_move_buffer(&self, move_buffer: &mut Vec<Move>) {
        move_buffer.clear();

        let empty_squares = self.board.unoccupied();
        let nonempty_squares = self.board.occupied();

        use Color::*;
        use PieceType::*;

        if self.to_move == White {

            let black_pieces = self.board.occupied_by(Black);

            /*********/
            /* PAWNS */
            /*********/

            let white_pawns = self.board.get_pieces(White, Pawn);
            let advanced_pawns = white_pawns.shifted_up();

            // single pushes (and promotions)
            for to in advanced_pawns & empty_squares
            {
                let from = Square::new(to.unwrap() - 8);

                if to.unwrap()/8 == 7 {
                    move_buffer.push(Move::new(from, to, BISHOP_PROMO_FLAG));
                    move_buffer.push(Move::new(from, to, KNIGHT_PROMO_FLAG));
                    move_buffer.push(Move::new(from, to, ROOK_PROMO_FLAG));
                    move_buffer.push(Move::new(from, to, QUEEN_PROMO_FLAG));
                } else {
                    move_buffer.push(Move::new(from, to, QUIET_FLAG));
                }
            }

            // double pushes
            for to in advanced_pawns.shifted_up() & empty_squares & RANK4 {
                let from = Square::new(to.unwrap() - 16);
                move_buffer.push(Move::new(from, to, DOUBLE_PAWN_PUSH_FLAG));
            }

            // captures (and capture-promotions)
            for from in white_pawns
            {
                for to in PAWN_ATTACKS[White as usize][from.idx()] & black_pieces
                {
                    if to.unwrap()/8 == 7 {
                        move_buffer.push(Move::new(from, to, BISHOP_PROMO_CAPTURE_FLAG));
                        move_buffer.push(Move::new(from, to, KNIGHT_PROMO_CAPTURE_FLAG));
                        move_buffer.push(Move::new(from, to, ROOK_PROMO_CAPTURE_FLAG));
                        move_buffer.push(Move::new(from, to, QUEEN_PROMO_CAPTURE_FLAG));
                    } else if self.ep_square.is_some() && self.ep_square.unwrap() == to {
                        move_buffer.push(Move::new(from, to, EP_CAPTURE_FLAG));
                    } else {
                        move_buffer.push(Move::new(from, to, CAPTURE_FLAG));
                    }
                }
            }

            /***********/
            /* KNIGHTS */
            /***********/

            for from in self.board.get_pieces(White, Knight)
            {
                let knight_moves = KNIGHT_TABLE[from.idx()];

                /* quiets */
                for to in knight_moves & empty_squares {
                    move_buffer.push(Move::new(from, to, QUIET_FLAG));
                }

                /* captures */
                for to in knight_moves & black_pieces {
                    move_buffer.push(Move::new(from, to, CAPTURE_FLAG));
                }

            }

            /***********/
            /* BISHOPS */
            /***********/

            for from in self.board.get_pieces(White, Bishop)
            {
                let bishop_moves = get_bishop_rays(from, nonempty_squares);

                /* quiets */
                for to in bishop_moves & empty_squares {
                    move_buffer.push(Move::new(from, to, QUIET_FLAG));
                }

                /* captures */
                for to in bishop_moves & black_pieces {
                    move_buffer.push(Move::new(from, to, CAPTURE_FLAG));
                }
            }

            /*********/
            /* ROOKS */
            /*********/

            for from in self.board.get_pieces(White, Rook)
            {
                let rook_moves = get_rook_rays(from, nonempty_squares);

                /* quiets */
                for to in rook_moves & empty_squares {
                    move_buffer.push(Move::new(from, to, QUIET_FLAG));
                }

                /* captures */
                for to in rook_moves & black_pieces {
                    move_buffer.push(Move::new(from, to, CAPTURE_FLAG));
                }
            }

            /*********/
            /* QUEEN */
            /*********/

            for from in self.board.get_pieces(White, Queen)
            {
                let queen_moves = get_queen_rays(from, nonempty_squares);

                /* quiets */
                for to in queen_moves & empty_squares {
                    move_buffer.push(Move::new(from, to, QUIET_FLAG));
                }

                /* captures */
                for to in queen_moves & black_pieces {
                    move_buffer.push(Move::new(from, to, CAPTURE_FLAG));
                }
            }

            /********/
            /* KING */
            /********/

            {
                let white_king_square = self.board.get_king_square(White);
                let king_moves = KING_TABLE[white_king_square.idx()];

                /* quiets */
                for to in king_moves & empty_squares {
                    if !self.board.is_square_attacked_by(to, Black) {
                        move_buffer.push(Move::new(white_king_square, to, QUIET_FLAG));
                    }
                }

                /* captures */
                for to in king_moves & black_pieces {
                    if !self.board.is_square_attacked_by(to, Black) {
                        move_buffer.push(Move::new(white_king_square, to, CAPTURE_FLAG));
                    }
                }

                /* castling */


                let has_kingside_castle_rights = self.castling_rights.intersects(CastlingRights::WHITE_KINGSIDE);
                let has_queenside_castle_rights = self.castling_rights.intersects(CastlingRights::WHITE_QUEENSIDE);
                let in_check = self.board.is_square_attacked_by(white_king_square, Black);

                if has_kingside_castle_rights && !in_check {
                    let kingside_castle_path_open = (nonempty_squares & WHITE_KINGSIDE_CASTLE_BITS).empty();

                    if kingside_castle_path_open {
                        let mut castle_path_is_safe: bool = true;

                        for sq in WHITE_KINGSIDE_CASTLE_BITS {
                            if self.board.is_square_attacked_by(sq, Black) {
                                castle_path_is_safe = false;
                            }
                        }

                        if castle_path_is_safe {
                            move_buffer.push(Move::new(white_king_square, Square::new(1), KING_CASTLE_FLAG));
                        }
                    }
                }

                if has_queenside_castle_rights && !in_check {
                    let queenside_castle_path_open = (nonempty_squares & WHITE_QUEENSIDE_CASTLE_BITS).empty();

                    if queenside_castle_path_open {
                        let mut castle_path_is_safe: bool = true;

                        for sq in WHITE_QUEENSIDE_CASTLE_BITS {
                            if self.board.is_square_attacked_by(sq, Black) {
                                castle_path_is_safe = false;
                            }
                        }

                        if castle_path_is_safe {
                            move_buffer.push(Move::new(white_king_square, Square::new(5), QUEEN_CASTLE_FLAG));
                        }
                    }
                }


            }



        }
    }
}

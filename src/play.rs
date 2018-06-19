use game::*;
use movegen::*;
use search::*;
use tree::*;
use eval::*;

// pub fn play_against_ai() {
//     // let mut tree = SearchTree::new(Game::from_fen_str("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1").unwrap());
//     let mut tree = SearchTree::new(Game::starting_position());
//
//     loop {
//         tree.focus().board.print();
//         println!("FEN: {}", tree.focus().to_fen());
//         println!("score: {}", Score::recompute(&tree.focus()).val);
//         println!("");
//         print!("Enter your move: ");
//
//         use std::io::{stdin,stdout,Write};
//         let _=stdout().flush();
//         let mut s=String::new();
//         stdin().read_line(&mut s).expect("Did not enter a correct string");
//
//         if let Some('\n')=s.chars().next_back() {
//             s.pop();
//         }
//         if let Some('\r')=s.chars().next_back() {
//             s.pop();
//         }
//
//         match move_from_algebraic(&tree.focus(), s) {
//             Some(m) => {
//                 tree.make_move(m);
//                 let game_copy = *tree.focus();
//                 tree.reset_root(game_copy, m);
//                 let (_, ai_move) = alpha_beta(&mut tree,6);
//                 tree.make_move(ai_move);
//             },
//             None => println!("Invalid move! Try again...")
//         }
//     }
//
// }

// use rand::{thread_rng, ThreadRng, Rng};

// pub struct MCTS {
//     move_gen: MoveGen,
//     rng: ThreadRng,
//     move_buffer: MoveBuffer
// }
//
// impl MCTS {
//     pub fn new() -> MCTS {
//         MCTS {
//             move_gen: MoveGen::new(),
//             rng: thread_rng(),
//             move_buffer: alloc_move_buffer()
//         }
//     }
//
//     pub fn play_random_game(&mut self, mut game: Game) -> GameResult {
//         while true {
//             self.move_gen.fill_move_buffer(&game, &self.move_buffer);
//             match game.outcome(self.move_buffer.borrow().len()) {
//                 Some(result) => {
//                     println!("{}", game.moves_played);
//                     return result;
//                 }
//                 None => {}
//             }
//
//             let num_moves = self.move_buffer.borrow().len();
//
//             let n = if (num_moves == 1) { 0 } else { self.rng.gen_range(0, num_moves - 1) };
//
//             game.make_move(self.move_buffer.borrow().at(n));
//         }
//
//         return GameResult::Draw;
//     }
// }


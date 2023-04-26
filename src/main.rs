// Simple terminal based Minesweeper with emoji graphics
// Uses terminal libary termion, which I found in the project https://github.com/isunjn/tic-tac-toe
// Navigate to directory and use the command 'cargo run'
// Instructions are in-game, change BOARD_WIDTH, BOARD_HEIGHT, BOMB_RATIO to change game difficulty
// Features recursive zero auto-clicking

extern crate termion;
extern crate rand;

use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

use std::io::{stdin, stdout, Write};
use std::usize;

use rand::seq::SliceRandom;
use rand::thread_rng;

// Global board variables
const BOARD_WIDTH: usize = 20;
const BOARD_HEIGHT: usize = 20;
const BOMB_RATIO: f64 = 0.15;

// Derived straight from other globals
const BOARD_SIZE: usize = BOARD_WIDTH*BOARD_HEIGHT;
const BOMB_NUM: usize = (BOARD_SIZE as f64 * BOMB_RATIO) as usize;

// Number emojis and spacing
const NUMBER_EMOJIS: [&str; 9] = [" 0 "," 1 "," 2 "," 3 "," 4 "," 5 "," 6 "," 7 "," 8 "];
const NUMBER_EMOJIS_HIGHLIGHT: [&str; 9] = ["0️⃣  ","1️⃣  ","2️⃣  ","3️⃣  ","4️⃣  ","5️⃣  ","6️⃣  ","7️⃣  ","8️⃣  "];

#[derive(Copy, Clone, PartialEq)]
enum SquareStatus {
    Unmodified,
    Flagged,
    Clicked,
}

#[derive(Copy, Clone, PartialEq)]
struct Square {
    bomb: bool,
    bomb_number: usize,
    status: SquareStatus
}

#[derive(Copy, Clone, PartialEq)]
enum GameStatus {
    Playing,
    Quit,
    Won,
    Lost,
}

struct Game {
    board: [Square; BOARD_SIZE],
    pos: usize,
    game_status: GameStatus,
    flags_remaining: u64,
    squares_remaining: usize
}

fn get_around_idxs(idx: usize) -> Vec<usize>{
    let mut res = Vec::new();
    let go_left = idx % BOARD_WIDTH > 0;
    let go_right = (idx + 1) % BOARD_WIDTH != 0;
    let go_up = idx / BOARD_WIDTH != 0;
    let go_down = idx / BOARD_WIDTH < BOARD_HEIGHT - 1;
    if go_left {
        if go_up {
            res.push(idx - BOARD_WIDTH - 1);
        }
        if go_down {
            res.push(idx + BOARD_WIDTH - 1);
        }
        res.push(idx - 1);
    }
    if go_right {
        if go_up {
            res.push(idx - BOARD_WIDTH + 1);
        }
        if go_down {
            res.push(idx + BOARD_WIDTH + 1);
        }
        res.push(idx + 1);
    }
    if go_up {
        res.push(idx - BOARD_WIDTH);
    }
    if go_down {
        res.push(idx + BOARD_WIDTH);
    }
    return res;
    
}


impl Game {
    fn new() -> Game {
        Game {
            board: [Square{bomb: false, bomb_number: 0, status: SquareStatus::Unmodified}; BOARD_SIZE],
            pos: 0,
            game_status: GameStatus::Playing,
            flags_remaining: BOMB_NUM as u64,
            squares_remaining: BOARD_SIZE,
        }
    }

    fn click_zeros(&mut self, idx: usize) -> () {
        if self.board[idx].bomb_number == 0 && self.board[idx].status == SquareStatus::Unmodified {
            self.board[idx].status = SquareStatus::Clicked;
            self.squares_remaining -= 1;
            let around_squares = get_around_idxs(idx);
            for i in around_squares {
                // Click all squares around, and if they are zeros recursively continue
                self.click_zeros(i);
            }
        } else if self.board[idx].status == SquareStatus::Unmodified {
            self.board[idx].status = SquareStatus::Clicked;
            self.squares_remaining -= 1;
        }
    }

    fn update_board(&mut self) {
        let pos = self.pos;
        loop {
            match stdin().keys().next().unwrap().unwrap() {
                Key::Char('q') => {
                    self.game_status = GameStatus::Quit;
                    break;
                }
                Key::Left => {
                    if self.pos % BOARD_WIDTH > 0 {
                        self.pos = (self.pos / BOARD_WIDTH) * BOARD_WIDTH + ((self.pos - 1) % BOARD_WIDTH);
                    }
                    break;
                }
                Key::Right => {
                    if BOARD_WIDTH - self.pos % BOARD_WIDTH > 1 {
                        self.pos = (self.pos / BOARD_WIDTH) * BOARD_WIDTH + ((self.pos + 1) % BOARD_WIDTH);
                    }
                    break;
                }
                Key::Up => {
                    if self.pos >= BOARD_WIDTH {
                        self.pos = self.pos - BOARD_WIDTH;
                    }
                    break;
                }
                Key::Down => {
                    if self.pos + BOARD_WIDTH < BOARD_SIZE {
                        self.pos = self.pos + BOARD_WIDTH;
                    }
                    break;
                }
                Key::Char('f') => {
                    // Flag if not clicked, or if flagged remove flag
                    // Not allowed to add more flags than bombs
                    if self.board[pos].status != SquareStatus::Clicked {
                        if self.board[pos].status == SquareStatus::Flagged {
                            self.board[pos].status = SquareStatus::Unmodified;
                            self.flags_remaining += 1;
                        } else if self.flags_remaining > 0 {   
                            self.board[pos].status = SquareStatus::Flagged;
                            self.flags_remaining -= 1;
                        }
                    }
                    break;
                }
                Key::Char('b') => {
                    // Click and reveal, or if flagged remove flag
                    if self.board[pos].status != SquareStatus::Clicked {
                        if self.board[pos].status == SquareStatus::Flagged {
                            self.board[pos].status = SquareStatus::Unmodified;
                            self.flags_remaining += 1;
                        } else {   
                            // If we click on a 0 bomb count square, recursively click squares around it
                            self.click_zeros(pos);
                            if self.squares_remaining == BOMB_NUM {
                                self.game_status = GameStatus::Won;
                            }
                            
                        }
                    }
                    // If we explode bomb, end game and lose
                    if self.board[pos].bomb {
                        self.game_status = GameStatus::Lost;
                    }
                    break;
                }
                _ => (),
            }
        }
    }

    fn draw_board(&self) {

        let instr: &str = "Press:\n\r'q' to quit\n\r'f' to flag/unflag\n\r'b' to click current square\n\rarrow keys to move\n\r";
        let info: String = format!(
            "number flags left to place: {}\n\r", self.flags_remaining
        );

        print!("{}{}", termion::clear::All, termion::cursor::Goto(1, 1));
        print!("\n\r  ────────── Minesweeper ──────────  \n\r\n\r"); 
        print!("{}{}", instr, info);

        for i in 0..BOARD_SIZE {
            if i % BOARD_WIDTH == 0 {
                print!("\n\r");
            }

            let mut emoji: &str = match self.board[i].status {
                SquareStatus::Flagged => "🚩 ",
                SquareStatus::Clicked => NUMBER_EMOJIS[self.board[i].bomb_number],
                SquareStatus::Unmodified => "⬛ ",
            };
            if i == self.pos {
                if self.board[i].bomb && self.board[i].status == SquareStatus::Clicked {
                    emoji = "💀 ";
                } else if self.board[i].status == SquareStatus::Clicked {
                    emoji = NUMBER_EMOJIS_HIGHLIGHT[self.board[i].bomb_number];
                } else {
                    emoji = "🟥 ";
                }
            }
            
            print!("{}", emoji);
            
        }
        print!("\n\r");

    }

    fn draw_end(&self) {
        self.draw_board();
        if self.game_status == GameStatus::Quit {
            print!("\n\r\x1b[34m               Thanks for playing! \n\r\n\r\x1b[0m");
        } else if self.game_status == GameStatus::Won {
            print!("\n\r\x1b[34m               You won 🎉🎉🎉\n\r\n\r\x1b[0m");
        } else if self.game_status == GameStatus::Lost {
            print!("\n\r\x1b[34m               You lost :( \n\r\n\r\x1b[0m");
        }
    }

}

fn main() {
    let mut stdout = stdout().into_raw_mode().unwrap();
    write!(stdout, "{}", termion::cursor::Hide).unwrap();
    let mut game = Game::new();

    // Set up bombs and bomb counts using random bits
    let mut idxs: Vec<usize> = (0..BOARD_SIZE).collect();
    let mut rng = thread_rng();
    idxs.shuffle(&mut rng);
    for i in 0..BOMB_NUM {
        let bomb_idx = idxs[i];
        game.board[bomb_idx].bomb = true;
        let around_squares = get_around_idxs(bomb_idx);
        for idx in around_squares {
            game.board[idx].bomb_number += 1;
        }
    }

    // Game loop 
    loop {
        game.draw_board();
        game.update_board();
        if game.game_status != GameStatus::Playing {
            game.draw_end();
            break;
        }
    }
    write!(stdout, "{}", termion::cursor::Show).unwrap();
}
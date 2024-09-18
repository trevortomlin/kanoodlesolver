// const BOARD_WIDTH: usize = 11;
// const BOARD_HEIGHT: usize = 5;

// type Board = [[bool; BOARD_WIDTH]; BOARD_HEIGHT];

// fn main() {
//     let mut board = [
//         [true; BOARD_WIDTH],
//         [true; BOARD_WIDTH],
//         [true; BOARD_WIDTH],
//         [true, true, true, true, true, true, true, true, false, false, false],
//         [true, true, true, true, true, true, true, true, true, false, false],
//     ];

//     let pieces: Vec<Vec<Vec<bool>>> = vec![
//         vec![
//             vec![true, true, true, true],
//             vec![true, false, false, false],
//         ],
//         vec![
//             vec![true, true, true],
//             vec![true, false, false],
//         ],
//         vec![
//             vec![false, true, false],
//             vec![true, true, true],
//             vec![false, true, false],
//         ],
//         vec![
//             vec![true, true],
//             vec![true, true],
//         ],
//         vec![
//             vec![true, false, true],
//             vec![true, true, true],
//         ],
//         vec![
//             vec![true, true, true, true],
//         ],
//         vec![
//             vec![false, false, false],
//             vec![false, true, true],
//             vec![true, true, false],
//         ],
//         vec![
//             vec![false, true],
//             vec![true, true],
//         ],
//         vec![
//             vec![false, true, true],
//             vec![true, true, true],
//         ],
//         vec![
//             vec![false, true, false, false],
//             vec![true, true, true, true],
//         ],
//         vec![
//             vec![true, true, true, false],
//             vec![false, false, true, true],
//         ],
//     ];

//     let mut all_orientations = vec![];
//     for piece in pieces {
//         let mut piece_orientations = vec![];
//         for rotation in 0..4 {
//             let rotated = rotate_piece(&piece, rotation);
//             piece_orientations.push(rotated.clone());
//             for flipped in 0..2 {
//                 let flipped = flip_piece(&rotated, flipped);
//                 piece_orientations.push(flipped);
//             }
//         }
//         all_orientations.push(piece_orientations);
//     }

//     if solve(&mut board, &all_orientations) {
//         println!("Solution found!");
//     } else {
//         println!("No solution exists.");
//     }
// }

// fn rotate_piece(piece: &Vec<Vec<bool>>, rotations: usize) -> Vec<Vec<bool>> {
//     let mut rotated = piece.clone();
//     for _ in 0..rotations {
//         rotated = rotate_90(&rotated);
//     }
//     rotated
// }

// fn rotate_90(piece: &Vec<Vec<bool>>) -> Vec<Vec<bool>> {
//     let rows = piece.len();
//     let cols = piece[0].len();
//     let mut rotated = vec![vec![false; rows]; cols];
//     for r in 0..rows {
//         for c in 0..cols {
//             rotated[c][rows - 1 - r] = piece[r][c];
//         }
//     }
//     rotated
// }

// fn flip_piece(piece: &Vec<Vec<bool>>, flip_type: usize) -> Vec<Vec<bool>> {
//     match flip_type {
//         0 => piece.clone(),  // No flip
//         1 => flip_horizontal(piece), // Horizontal flip
//         _ => flip_vertical(piece), // Vertical flip
//     }
// }

// fn flip_horizontal(piece: &Vec<Vec<bool>>) -> Vec<Vec<bool>> {
//     piece.iter().map(|row| row.iter().rev().cloned().collect()).collect()
// }

// fn flip_vertical(piece: &Vec<Vec<bool>>) -> Vec<Vec<bool>> {
//     piece.iter().rev().cloned().collect()
// }

// fn solve(board: &mut Board, pieces: &[Vec<Vec<Vec<bool>>>]) -> bool {
//     if is_filled(board) {
//         return true;
//     }

//     for piece_orientations in pieces {
//         for piece_shape in piece_orientations {
//             for y in 0..=BOARD_HEIGHT - piece_shape.len() {
//                 for x in 0..=BOARD_WIDTH - piece_shape[0].len() {
//                     if can_place_piece(board, &piece_shape, x, y) {
//                         place_piece(board, &piece_shape, x, y);
//                         if solve(board, pieces) {
//                             return true;
//                         }
//                         remove_piece(board, &piece_shape, x, y);
//                     }
//                 }
//             }
//         }
//     }
//     false
// }

// fn is_filled(board: &Board) -> bool {
//     for row in board {
//         if row.contains(&false) {
//             return false;
//         }
//     }
//     true
// }

// fn can_place_piece(board: &Board, piece: &Vec<Vec<bool>>, x: usize, y: usize) -> bool {
//     for (dy, row) in piece.iter().enumerate() {
//         for (dx, &cell) in row.iter().enumerate() {
//             if cell {
//                 let board_x = x + dx;
//                 let board_y = y + dy;
//                 if board_x >= BOARD_WIDTH || board_y >= BOARD_HEIGHT || board[board_y][board_x] {
//                     return false;
//                 }
//             }
//         }
//     }
//     true
// }

// fn place_piece(board: &mut Board, piece: &Vec<Vec<bool>>, x: usize, y: usize) {
//     for (dy, row) in piece.iter().enumerate() {
//         for (dx, &cell) in row.iter().enumerate() {
//             if cell {
//                 board[y + dy][x + dx] = true;
//             }
//         }
//     }
// }

// fn remove_piece(board: &mut Board, piece: &Vec<Vec<bool>>, x: usize, y: usize) {
//     for (dy, row) in piece.iter().enumerate() {
//         for (dx, &cell) in row.iter().enumerate() {
//             if cell {
//                 board[y + dy][x + dx] = false;
//             }
//         }
//     }
// }

use ansi_term::Colour;
use serde::Deserialize;
use serde_json::{self, Value};
use std::fs;

const PRINT_CHAR: &str = "●";
const GRID_WIDTH: usize = 11;
const GRID_HEIGHT: usize = 5;


#[derive(Deserialize, Debug)]
struct Puzzle {
    pieces: Vec<Piece>,
}

#[derive(Deserialize, Debug)]
struct Piece {
    piece: String,
    x: usize,
    y: usize,
    transformation: Transformation,
}

#[derive(Deserialize, Debug)]
struct Transformation {
    rotation: usize,
    flip_horizontal: bool,
    flip_vertical: bool,
    shape: Vec<Vec<bool>>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data = fs::read_to_string("json/puzzle_config.json")?;

    let puzzles: Value = serde_json::from_str(&data)?;
    
    let puzzle_number = "0";
    
    if let Some(puzzle_data) = puzzles.get(puzzle_number) {
        let pieces: Vec<Piece> = serde_json::from_value(puzzle_data.clone())?;

        let mut grid = vec![vec![None; 11]; 5];
        
        for piece in pieces {
            place_piece(&mut grid, &piece);
        }
        
        for row in grid {
            for cell in row {
                match cell {
                    Some(ref color) => print!("{} ", color_square(color)),
                    None => print!("{} ", color_square("dark_gray")),
                }
            }
            println!();
        }
    } else {
        println!("Puzzle not found!");
    }

    Ok(())
}

fn place_piece(grid: &mut Vec<Vec<Option<String>>>, piece: &Piece) {
    let shape = &piece.transformation.shape;
    let start_x = piece.x;
    let start_y = piece.y;

    for (i, row) in shape.iter().enumerate() {
        for (j, &cell) in row.iter().enumerate() {
            if cell {
                let x = start_x + j;
                let y = start_y + i;

                if y < 5 && x < 11 {
                    grid[y][x] = Some(piece.piece.clone());
                }
            }
        }
    }
}

fn color_square(color: &str) -> String {
    match color {
        "blue" => Colour::Blue.paint(PRINT_CHAR).to_string(),
        "red" => Colour::Red.paint(PRINT_CHAR).to_string(),
        "green" => Colour::Green.paint(PRINT_CHAR).to_string(),
        "yellow" => Colour::Yellow.paint(PRINT_CHAR).to_string(),
        "cyan" => Colour::Cyan.paint(PRINT_CHAR).to_string(),
        "purple" => Colour::Purple.paint(PRINT_CHAR).to_string(),
        "magenta" => Colour::RGB(252, 92, 172).paint(PRINT_CHAR).to_string(),
        "dark_gray" => Colour::RGB(101, 102, 107).paint(PRINT_CHAR).to_string(),
        "white" => Colour::White.paint(PRINT_CHAR).to_string(),
        "pink" => Colour::RGB(255, 213, 206).paint(PRINT_CHAR).to_string(),
        "orange" => Colour::RGB(255, 125, 36).paint(PRINT_CHAR).to_string(),
        "yellow_green" => Colour::RGB(190, 214, 67).paint(PRINT_CHAR).to_string(),
        "off_white" => Colour::RGB(247, 243, 227).paint(PRINT_CHAR).to_string(),
        "light_gray" => Colour::RGB(176, 177, 179).paint(PRINT_CHAR).to_string(),
        _ => Colour::White.paint(PRINT_CHAR).to_string(), // Default to white if unknown
    }
}

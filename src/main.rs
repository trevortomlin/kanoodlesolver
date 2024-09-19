use ansi_term::Colour;
use serde::Deserialize;
use serde_json::{self, Value};
use std::{collections::HashMap, fs};

const PRINT_CHAR: &str = "●";
const GRID_WIDTH: usize = 11;
const GRID_HEIGHT: usize = 5;

#[derive(Deserialize, Debug)]
struct Puzzle {
    pieces: Vec<CurrentPiece>,
}

#[derive(Deserialize, Debug)]
struct CurrentPiece {
    piece: String,
    x: usize,
    y: usize,
    transformation: Transformation,
}

#[derive(Deserialize, Debug)]
struct PossiblePiece {
    piece: String,
    all_transformations: Vec<Transformation>,
}

#[derive(Deserialize, Debug, Clone)]
struct Transformation {
    rotation: usize,
    flip_horizontal: bool,
    flip_vertical: bool,
    shape: Vec<Vec<bool>>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data = fs::read_to_string("json/puzzle_config.json")?;

    let transform_data = fs::read_to_string("json/shapes_transformations.json")?;
    let transformation_map: HashMap<String, Vec<Transformation>> = serde_json::from_str(&transform_data)?;

    let possible_pieces: Vec<PossiblePiece> = transformation_map.into_iter()
        .map(|(piece_name, transformations)| PossiblePiece {
            piece: piece_name,
            all_transformations: transformations,
        })
        .collect();

    let puzzles: Value = serde_json::from_str(&data)?;

    let mut solved = 0;

    for p in 0..=161 {
        println!("Puzzle: {}", p);
        if let Some(puzzle_data) = puzzles.get(p.to_string()) {
            let current_pieces: Vec<CurrentPiece> = serde_json::from_value(puzzle_data.clone())?;

            let mut grid = vec![vec![None; GRID_WIDTH]; GRID_HEIGHT];

            for piece in &current_pieces {
                place_piece(&mut grid, &piece.transformation.shape, piece.x, piece.y, true, &piece.piece);
            }

            // print_grid(&grid);

            println!();

            let mut found_solution = false;

            let len = possible_pieces.len();
            for i in 0..len {
                let mut new_grid = grid.clone();
                if place_pieces_backtrack(&mut new_grid, &possible_pieces, i) {
                    grid = new_grid;
                    found_solution = true;
                    // print_grid(&grid);
                    solved += 1;
                    break;
                }
            } 
            
            if !found_solution {
                println!("No solution found!");
            }
        } else {
            println!("Puzzle not found!");
        }

    }

    println!("Solved {} out of 161", solved);

    Ok(())
}

fn print_grid(grid: &Vec<Vec<Option<String>>>) {
    for row in grid {
        for cell in row {
            match cell {
                Some(piece_name) => print!("{} ", color_square(piece_name)),
                None => print!("{} ", color_square("dark_gray")),
            }
        }
        println!();
    }
}

fn place_pieces_backtrack(
    grid: &mut Vec<Vec<Option<String>>>,
    pieces: &[PossiblePiece],
    index: usize,
) -> bool {

    if index == pieces.len() {
        return false;
    }
    if is_grid_filled(grid) {
        return true;
    }

    let piece = &pieces[index];

    for transformation in &piece.all_transformations {
        let shape = &transformation.shape;

        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                if can_place_piece(grid, shape, x, y) {
                    place_piece(grid, shape, x, y, true, &piece.piece);

                    if place_pieces_backtrack(grid, pieces, index + 1) {
                        return true;
                    }

                    place_piece(grid, shape, x, y, false, &piece.piece);
                }
            }
        }
    }

    false
}

fn can_place_piece(
    grid: &Vec<Vec<Option<String>>>,
    shape: &Vec<Vec<bool>>,
    start_x: usize,
    start_y: usize,
) -> bool {
    for (i, row) in shape.iter().enumerate() {
        for (j, &cell) in row.iter().enumerate() {
            if cell {
                let x = start_x + j;
                let y = start_y + i;

                if y >= GRID_HEIGHT || x >= GRID_WIDTH || grid[y][x].is_some() {
                    return false;
                }
            }
        }
    }
    true
}

fn place_piece(grid: &mut Vec<Vec<Option<String>>>, shape: &Vec<Vec<bool>>, x: usize, y: usize, place: bool, piece_name: &str) {
    for (i, row) in shape.iter().enumerate() {
        for (j, &cell) in row.iter().enumerate() {
            if cell {
                let grid_x = x + j;
                let grid_y = y + i;

                if grid_y < GRID_HEIGHT && grid_x < GRID_WIDTH {
                    if place {
                        grid[grid_y][grid_x] = Some(piece_name.to_string());
                    } else {
                        grid[grid_y][grid_x] = Some("dark_gray".to_string());
                    }
                }
            }
        }
    }
}

fn is_grid_filled(grid: &Vec<Vec<Option<String>>>) -> bool {
    for row in grid {
        for cell in row {
            if cell.is_none() {
                return false;
            }
        }
    }
    true 
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
        a => {
            println!("{}", a);
            Colour::White.paint(PRINT_CHAR).to_string()
        }
    }
}

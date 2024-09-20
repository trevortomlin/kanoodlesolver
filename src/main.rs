use ansi_term::Colour;
use serde::Deserialize;
use serde_json::{self, Value};
use std::{collections::{HashMap, HashSet}, fs, ops::Div, time::{Duration, Instant}};
use rayon::prelude::*;
use std::sync::Arc;

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

#[derive(Deserialize, Debug, Clone)]
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

    let (pieces, puzzles) = load_data()?;

    let num_trials: usize = 5;

    let mut st_average = Duration::new(0, 0);
    for _ in 0..num_trials {
        let before = Instant::now();
        let _ = single_thread_solve(&pieces, &puzzles);
        st_average += before.elapsed();
    }
    println!("Single-threaded average over {} trials: {:.2?}s", num_trials, st_average.div_f32(num_trials as f32));

    let mut st_average = Duration::new(0, 0);
    for _ in 0..num_trials {
        let before = Instant::now();
        let _ = multi_threaded_solve(&pieces, &puzzles);
        st_average += before.elapsed();
    }
    println!("Multi-threaded average over {} trials: {:.2?}s", num_trials, st_average.div_f32(num_trials as f32));
    
    Ok(())
}

fn load_data() -> Result<(Vec<PossiblePiece>, Value), Box<dyn std::error::Error>> {
    let data = fs::read_to_string("json/puzzle_config.json")?;
    let transform_data = fs::read_to_string("json/shapes_transformations.json")?;
    let transformation_map: HashMap<String, Vec<Transformation>> = serde_json::from_str(&transform_data)?;

    let pieces: Vec<PossiblePiece> = transformation_map.into_iter()
        .map(|(piece_name, transformations)| PossiblePiece {
            piece: piece_name,
            all_transformations: transformations,
        })
        .collect();

    let puzzles: Value = serde_json::from_str(&data)?;
    Ok((pieces, puzzles))
}

fn single_thread_solve(possible_pieces: &Vec<PossiblePiece>, puzzles: &Value) ->  Result<(), Box<dyn std::error::Error>> {
    let mut solved = 0;

    for p in 0..=161 {
        // println!("Puzzle: {}", p);
        if let Some(puzzle_data) = puzzles.get(p.to_string()) {
            let current_pieces: Vec<CurrentPiece> = serde_json::from_value(puzzle_data.clone())?;
            let mut grid = vec![vec!["dark_gray".to_string(); GRID_WIDTH]; GRID_HEIGHT];

            for piece in &current_pieces {
                place_piece(&mut grid, &piece.transformation.shape, piece.x, piece.y, true, &piece.piece);
            }

            let mut found_solution = false;

            let current_piece_set: HashSet<String> = current_pieces.iter()
                .map(|l| l.piece.clone())
                .collect();

            let possible_pieces: Vec<PossiblePiece> = possible_pieces.iter()
                .filter(|e| !current_piece_set.contains(&e.piece))
                .cloned() 
                .collect();

            let mut used_pieces = HashSet::new();

            if place_pieces_backtrack(&mut grid, &possible_pieces, 0, &mut used_pieces) {
                found_solution = true;
                solved += 1;
            }

            if !found_solution {
                println!("No solution found!");
            }
        } else {
            println!("Puzzle {} not found!", p);
        }
    }

    // println!("Solved {} out of 162", solved);
    Ok(())
}

fn multi_threaded_solve(possible_pieces: &Vec<PossiblePiece>, puzzles: &Value) -> Result<(), Box<dyn std::error::Error>> {
    let num_cpus = num_cpus::get();
    let puzzle_nums: Vec<_> = (0..=161).collect();
    let possible_pieces = Arc::new(possible_pieces);

    let solved_count = puzzle_nums.par_chunks(num_cpus).into_par_iter().filter_map(|puzzle_batch| {
        let possible_pieces = Arc::clone(&possible_pieces);
        
        let mut batch_solved = 0;

        for &p in puzzle_batch {
            if let Some(puzzle_data) = puzzles.get(p.to_string()) {
                let current_pieces: Vec<CurrentPiece> = serde_json::from_value(puzzle_data.clone()).unwrap();
                let mut grid = vec![vec!["dark_gray".to_string(); GRID_WIDTH]; GRID_HEIGHT];

                for piece in &current_pieces {
                    place_piece(&mut grid, &piece.transformation.shape, piece.x, piece.y, true, &piece.piece);
                }

                let current_piece_set: HashSet<String> = current_pieces.iter()
                    .map(|l| l.piece.clone())
                    .collect();

                let remaining_pieces: Vec<PossiblePiece> = possible_pieces.iter()
                    .filter(|e| !current_piece_set.contains(&e.piece))
                    .cloned()
                    .collect();

                let mut used_pieces = HashSet::new();
                if place_pieces_backtrack(&mut grid, &remaining_pieces, 0, &mut used_pieces) {
                    batch_solved += 1;
                }
            } else {
                println!("Puzzle {} not found!", p);
            }
        }
        Some(batch_solved)
    }).sum::<usize>();

    // println!("Solved {} out of 162", solved_count);
    Ok(())
}


fn place_pieces_backtrack(
    grid: &mut Vec<Vec<String>>,
    pieces: &[PossiblePiece],
    index: usize,
    used_pieces: &mut HashSet<String>,
) -> bool {
    if index == pieces.len() {
        return is_grid_filled(grid);
    }

    let piece = &pieces[index];

    if used_pieces.contains(&piece.piece) {
        return place_pieces_backtrack(grid, pieces, index + 1, used_pieces);
    }

    for transformation in &piece.all_transformations {
        let shape = &transformation.shape;

        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                if can_place_piece(grid, shape, x, y) {
                    place_piece(grid, shape, x, y, true, &piece.piece);
                    used_pieces.insert(piece.piece.clone());

                    if place_pieces_backtrack(grid, pieces, index + 1, used_pieces) {
                        return true;
                    }

                    place_piece(grid, shape, x, y, false, &piece.piece);
                    used_pieces.remove(&piece.piece);
                }
            }
        }
    }

    false
}

fn can_place_piece(
    grid: &Vec<Vec<String>>,
    shape: &Vec<Vec<bool>>,
    start_x: usize,
    start_y: usize,
) -> bool {
    for (i, row) in shape.iter().enumerate() {
        for (j, &cell) in row.iter().enumerate() {
            if cell {
                let x = start_x + j;
                let y = start_y + i;

                if y >= GRID_HEIGHT || x >= GRID_WIDTH || grid[y][x] != "dark_gray" {
                    return false;
                }
            }
        }
    }
    true
}

fn place_piece(grid: &mut Vec<Vec<String>>, shape: &Vec<Vec<bool>>, x: usize, y: usize, place: bool, piece_name: &str) {
    for (i, row) in shape.iter().enumerate() {
        for (j, &cell) in row.iter().enumerate() {
            if cell {
                let grid_x = x + j;
                let grid_y = y + i;

                if grid_y < GRID_HEIGHT && grid_x < GRID_WIDTH {
                    if place {
                        grid[grid_y][grid_x] = piece_name.to_string();
                    } else {
                        grid[grid_y][grid_x] = "dark_gray".to_string();
                    }
                }
            }
        }
    }
}

fn is_grid_filled(grid: &Vec<Vec<String>>) -> bool {
    for row in grid {
        for cell in row {
            if cell == "dark_gray" {
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

fn print_grid(grid: &Vec<Vec<String>>) {
    for row in grid {
        for cell in row {
            print!("{} ", color_square(cell));
        }
        println!();
    }
}
use macroquad::prelude::*;
use macroquad::audio::{Sound, load_sound, play_sound, PlaySoundParams};

const BLOCK: f32 = 30.0;
const ROWS: usize = 20;
const COLS: usize = 10;
const DROP_TIME: f32 = 0.5;
const SCORE_PER_LINE: i32 = 1;
const PIECES: &[&[&[u8]]] = &[
    // O
    &[&[1,1],
    &[1,1]],
    // I
    &[&[1,1,1,1]],
    // T
    &[&[1,0,0],
    &[1,1,1]],
    // L
    &[&[1,0,0],
    &[1,1,1]],
    // J
    &[&[0,0,1],
    &[1,1,1]],
    // S
    &[&[0,0,1],
    &[1,1,0]],
    // Z
    &[&[1,1,0],
    &[0,1,1]],
];
const COLORS: &[Color] = &[
    RED,
    GREEN,
    BLUE,
    YELLOW,
    ORANGE,
    PINK,
    PURPLE,
    SKYBLUE,
];

type Grid = [[u8; COLS]; ROWS];
type ColorGrid = [[Option<Color>; COLS]; ROWS];

#[derive(Clone)]
struct Piece {
    shape: Vec<Vec<u8>>,
    x: i32,
    y: i32,
    timer: f32,
    color: Color,
}

struct SimpleRng {
    state: u32,
}

impl SimpleRng {
    fn new(seed: u32) -> Self {
        Self { state: seed }
    }

    fn next(&mut self) -> u32 {
        self.state = self.state.wrapping_mul(1664525).wrapping_add(1013904223);
        self.state
    }

    fn range(&mut self, min: usize, max: usize) -> usize {
        (self.next() as usize % (max - min)) + min
    }
}

fn new_piece(rng: &mut SimpleRng) -> Piece {
    let index = rng.range(0, PIECES.len());
    let shape_ref = PIECES[index];
    let shape: Vec<Vec<u8>> = shape_ref.iter().map(|row| row.to_vec()).collect();

    let color_index = rng.range(0, COLORS.len());
    let color = COLORS[color_index];

    Piece {
        shape,
        x: 4,
        y: 0,
        timer: 0.0,
        color,
    }
}

fn rotate(piece: &mut Piece) {
    let h = piece.shape.len();
    let w = piece.shape[0].len();
    let mut new_shape = vec![vec![0; h]; w];
    for y in 0..h {
        for x in 0..w {
            new_shape[x][h - 1 - y] = piece.shape[y][x];
        }
    }
    piece.shape = new_shape;
}

fn collide(grid: &Grid, piece: &Piece, dx: i32, dy: i32) -> bool {
    for (y, row) in piece.shape.iter().enumerate() {
        for (x, &cell) in row.iter().enumerate() {
            if cell == 0{
                continue;
            }

            let nx = piece.x + x as i32 + dx;
            let ny = piece.y + y as i32 + dy;

            if nx < 0 || nx >= COLS as i32 || ny >= ROWS as i32 {
                return true;
            }
            if ny >= 0 && grid[ny as usize ][nx as usize] == 1 {
                return true;
            }
        }
    }
    false
}

fn lock_piece(grid: &mut Grid, color_grid: &mut ColorGrid, piece: &Piece) {
    for (y, row) in piece.shape.iter().enumerate() {
        for (x, &cell) in row.iter().enumerate() {
            if cell == 1 {
                let gx = (piece.x + x as i32) as usize;
                let gy = (piece.y + y as i32) as usize;
                grid[gy][gx] = 1;
                color_grid[gy][gx] = Some(piece.color);
            }
        }
    }
}

fn clear_lines(grid: &mut Grid, color_grid: &mut ColorGrid) -> i32 {
    let mut new_grid = [[0u8; COLS]; ROWS];
    let mut new_color_grid: ColorGrid = [[None; COLS]; ROWS];
    let mut new_y = ROWS as i32 - 1;
    let mut lines_cleared = 0;

    for y in (0..ROWS).rev() {
        if grid[y].iter().all(|&c| c == 1) {
            lines_cleared += 1;
            continue;
        }
        new_grid[new_y as usize] = grid[y];
        new_color_grid[new_y as usize] = color_grid[y];
        new_y -= 1
    }
     *grid = new_grid;
     *color_grid = new_color_grid;

     lines_cleared
}

#[macroquad::main(window_conf)]

async fn main() {

    let move_sound: Sound = load_sound("audio/Move.wav").await.unwrap();
    let locks_sound: Sound = load_sound("audio/Lock.wav").await.unwrap();
    let gameover_sound: Sound = load_sound("audio/GameOver.wav").await.unwrap();
    let lineclear_sound: Sound = load_sound("audio/LineClear.wav").await.unwrap();


    let mut rng = SimpleRng::new(42);
    
    let mut grid: Grid = [[0; COLS]; ROWS];
    let mut piece = new_piece(&mut rng);
    let mut score: i32 = 0;
    let mut color_grid: ColorGrid = [[None; COLS]; ROWS];
    
    loop {
        let dt = get_frame_time();
        piece.timer += dt;

        if piece.timer > DROP_TIME {
            if collide(&grid, &piece, 0, 1) {
                play_sound(&locks_sound, PlaySoundParams { volume: 1.0, looped: false });
                lock_piece(&mut grid, &mut color_grid, &piece);
                let lines = clear_lines(&mut grid, &mut color_grid);
                score += lines * SCORE_PER_LINE;

                piece = new_piece(&mut rng);

                if collide(&grid, &piece, 0, 0) {
                    for _ in 0..120 {
                        clear_background(BLACK);
                        play_sound(&gameover_sound, PlaySoundParams { volume: 1.0, looped: false });
                        draw_text(
                            &format!("Game OVER!"),
                            50.0,
                            300.0,
                            40.0,
                            RED,
                        );
                    next_frame().await;
                    }
                    grid = [[0; COLS]; ROWS];
                    score = 0;
                    piece = new_piece(&mut rng);
                }
            } else {
                piece.y += 1;
            }

            piece.timer = 0.0;
        }

        clear_background(BLACK);

        if is_key_pressed(KeyCode::Left) {
            if !collide(&grid, &piece, -1, 0) {
                piece.x -= 1;
                play_sound(&move_sound, PlaySoundParams { volume: 1.0, looped: false });
            }
        }
        if is_key_pressed(KeyCode::Right) {
            if !collide(&grid, &piece, 1, 0) {
                piece.x += 1;
                play_sound(&move_sound, PlaySoundParams { volume: 1.0, looped: false });
            }
        }
        if is_key_pressed(KeyCode::Down) {
            if !collide(&grid, &piece, 0, 1) {
                piece.y += 1;
                play_sound(&move_sound, PlaySoundParams { volume: 1.0, looped: false });
            } else {
                lock_piece(&mut grid, &mut color_grid, &piece);
                let lines = clear_lines(&mut grid, &mut color_grid);
                score += lines * SCORE_PER_LINE;
                play_sound(&lineclear_sound, PlaySoundParams { volume: 1.0, looped: false });
                piece = new_piece(&mut rng);
            }
        }
        if is_key_pressed(KeyCode::Up) {
            let mut rotated_piece = piece.clone();
            rotate(&mut rotated_piece);
            if !collide(&grid, &rotated_piece, 0, 0) {
                piece = rotated_piece;
                play_sound(&move_sound, PlaySoundParams { volume: 1.0, looped: false });
            }
        }

        for y in 0..ROWS {
            for x in 0..COLS {
                if grid[y][x] == 1 {
                    if let Some(color) = color_grid[y][x] {
                        draw_rectangle(
                            x as f32 * BLOCK,
                            y as f32 * BLOCK,
                            BLOCK,
                            BLOCK,
                            color,
                        );
                    }
                }
            }
        }

        for (y, row) in piece.shape.iter().enumerate() {
            for (x, &cell) in row.iter().enumerate() {
                if cell == 1 {
                    draw_rectangle(
                        (piece.x + x as i32) as f32 * BLOCK,
                        (piece.y + y as i32) as f32 * BLOCK,
                        BLOCK,
                        BLOCK,
                        piece.color,
                    );
                }
            }
        }

        for y in 0..ROWS {
            for x in 0..COLS {
                draw_rectangle_lines(
                    x as f32 * BLOCK,
                    y as f32 * BLOCK,
                    BLOCK,
                    BLOCK,
                    1.0,
                    GRAY,
                );
            }
        }

        draw_text(
            &format!("SCORE : {}", score),
            10.0,
            20.0,
            30.0,
            WHITE,
        );

        next_frame().await;
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Tetris".to_string(),
        window_width: 300,
        window_height: 600,
        ..Default::default()
    }
}
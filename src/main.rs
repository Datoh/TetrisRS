use std::time::Duration;

use ggez;
use ggez::conf;
use ggez::event;
use ggez::graphics;
use ggez::nalgebra as na;
use ggez::timer;
use ggez::{Context, GameResult};

use rand::{ distributions::{Distribution, Standard}, Rng};

const GRID_WIDTH: usize = 10;
const GRID_HEIGHT: usize = 20;

#[derive(Clone,Copy,PartialEq)]
enum Case {
  Empty,
  Red,
  Green,
  Blue,
  Yellow,
  DarkYellow,
  Purple,
  Cyan,
}

impl Distribution<Case> for Standard {
  fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Case {
    match rng.gen_range(1, 8) {
      1 => Case::Red,
      2 => Case::Green,
      3 => Case::Blue,
      4 => Case::Yellow,
      5 => Case::DarkYellow,
      6 => Case::Purple,
      7 => Case::Cyan,
      _ => Case::Empty,
    }
  }
}
const CASE_SIZE:   f32 = 20.0;
const CASE_BORDER: f32 = 2.0;

fn case_color(case: Case) -> graphics::Color {
  return match case {
    Case::Red => graphics::Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 },
    Case::Green => graphics::Color { r: 0.0, g: 1.0, b: 0.0, a: 1.0 },
    Case::Blue => graphics::Color { r: 0.0, g: 0.0, b: 1.0, a: 1.0 },
    Case::Yellow => graphics::Color { r: 1.0, g: 1.0, b: 0.0, a: 1.0 },
    Case::DarkYellow => graphics::Color { r: 1.0, g: 0.85, b: 0.0, a: 1.0 },
    Case::Purple => graphics::Color { r: 0.5, g: 0.0, b: 0.5, a: 1.0 },
    Case::Cyan => graphics::Color { r: 0.0, g: 1.0, b: 1.0, a: 1.0 },
    _ => panic!("Unknow case type"),
  };
}

#[derive(Clone,Copy)]
struct Offset {
  x: i32,
  y: i32,
}
const ROTATION_OFFSET_DEFAULT: [Offset; 4] = [Offset { x: 1, y: 0}, Offset { x: -1, y: 1}, Offset { x: 0, y: -1}, Offset { x: 0, y: 0}, ];
const ROTATION_OFFSET_CYAN: [Offset; 4] = [Offset { x: 2, y: -1}, Offset { x: -2, y: 2}, Offset { x: 1, y: -2}, Offset { x: -1, y: 1}, ];

fn cases_rotation_offset(case: Case, index: usize) -> Offset {
  return match case {
    Case::DarkYellow => Offset { x: 0, y: 0},
    Case::Cyan => ROTATION_OFFSET_CYAN[index],
    _ => ROTATION_OFFSET_DEFAULT[index],
  };
}

fn piece_cases(case: Case) -> Vec<Vec<Case>> {
  return match case {
    Case::Red => vec![
      vec![Case::Red, Case::Red, Case::Empty], 
      vec![Case::Empty, Case::Red, Case::Red],
    ],
    Case::Green => vec![
      vec![Case::Empty, Case::Green, Case::Green],
      vec![Case::Green, Case::Green, Case::Empty], 
    ],
    Case::Blue => vec![
      vec![Case::Blue, Case::Empty, Case::Empty],
      vec![Case::Blue, Case::Blue, Case::Blue], 
    ],
    Case::Yellow => vec![
      vec![Case::Empty, Case::Empty, Case::Yellow],
      vec![Case::Yellow, Case::Yellow, Case::Yellow], 
    ],
    Case::DarkYellow => vec![
      vec![Case::DarkYellow, Case::DarkYellow],
      vec![Case::DarkYellow, Case::DarkYellow], 
    ],
    Case::Purple => vec![
      vec![Case::Empty, Case::Purple, Case::Empty],
      vec![Case::Purple, Case::Purple, Case::Purple], 
    ],
    Case::Cyan => vec![
      vec![Case::Cyan, Case::Cyan, Case::Cyan, Case::Cyan],
    ],
    _ => panic!("Unknow case type"),
  };
}

struct Piece {
  case: Case,
  x: i32,
  y: i32,
  last_move: Duration,
  cases: Vec<Vec<Case>>,
  index_rotation: usize,
}

impl Piece {
  fn width(&self) -> i32 {
    return self.cases[0].len() as i32;
  }
  fn height(&self) -> i32 {
    return self.cases.len() as i32;
  }
}

fn create_piece(case: Case) -> Piece {
  let cases = piece_cases(case);
  return Piece { case: case, x: ((GRID_WIDTH - cases[0].len()) / 2) as i32, y: 0, last_move: Duration::from_secs(0), cases: cases, index_rotation: 0 };
}

fn check_collision(grid: &[[Case; GRID_HEIGHT]; GRID_WIDTH], piece: &Piece, dx: i32, dy: i32) -> bool {
  let piece_x = piece.x + dx;
  let piece_y = piece.y + dy;

  if piece_y + piece.height() > GRID_HEIGHT as i32 {
    return true;
  }
  if piece_x < 0 || piece_x + piece.width() > GRID_WIDTH as i32 {
    return true;
  }

  for (i_v_y, line) in piece.cases.iter().enumerate() {
    let i_y = piece_y as usize + i_v_y;
    for (i_v_x, &case) in line.iter().enumerate() {
      if case != Case::Empty {
        let i_x = piece_x as usize + i_v_x;
        if grid[i_x][i_y] != Case::Empty {
          return true;
        }
      }
    }
  }

  return false;
}

fn drop_speed(level: u32) -> Duration {
  let level_f64 = (level - 1) as f64;
  Duration::from_secs_f64((0.8 - (level_f64 * 0.007)).powf(level_f64))
}

struct MainState {
  grid: [[Case; GRID_HEIGHT]; GRID_WIDTH],
  grid_rect: graphics::Rect,
  current_piece: Option<Piece>,
  move_speed: Duration,
  timer_piece_generation: Duration,
  score: i64,
  level: u32,
  line_removed: u32,
}

impl MainState {
  fn new() -> GameResult<MainState> {
    let width = (GRID_WIDTH as f32) * (CASE_SIZE + 2.0 * CASE_BORDER);
    let height = (GRID_HEIGHT as f32) * (CASE_SIZE + 2.0 * CASE_BORDER);
    let left = (800.0 - width) / 2.0;
    let top = (600.0 - height) / 2.0;

    let s = MainState {
      grid: [[Case::Empty; GRID_HEIGHT]; GRID_WIDTH],
      grid_rect: graphics::Rect::new(left, top, width, height),
      current_piece: None,
      move_speed: drop_speed(1),
      timer_piece_generation: Duration::from_secs(0),
      score: 0,
      level: 1,
      line_removed: 0,
    };
    Ok(s)
  }

  fn rotate(&mut self) {
    if self.current_piece.is_none() {
      return;
    }

    let old_piece = self.current_piece.as_ref().unwrap();
    let mut tmp_cases: Vec<Vec<Case>> = Vec::new();
    let height = old_piece.cases.len();
    let width = old_piece.cases[0].len();
    for x in 0..width {
      let mut current_row: Vec<Case> = Vec::new();
      for y in 0..height {
        current_row.push(old_piece.cases[y][x]);
      }
      current_row.reverse();
      tmp_cases.push(current_row);
    }
    let mut piece = Piece { case: old_piece.case, x: old_piece.x, y: old_piece.y, last_move: old_piece.last_move, cases: tmp_cases, index_rotation: old_piece.index_rotation };
    let offset = cases_rotation_offset(piece.case, piece.index_rotation);
    piece.x += offset.x;
    piece.y += offset.y;
    piece.y = piece.y.max(0);
    piece.index_rotation = (piece.index_rotation + 1) % 4;

    let mut ok = !check_collision(&self.grid, &piece, 0, 0);
    if !ok {
      piece.x -= 1;
      ok = !check_collision(&self.grid, &piece, 0, 0);
    }
    if !ok {
      piece.x += 2;
      ok = !check_collision(&self.grid, &piece, 0, 0);
    }
    if !ok {
      piece.x -= 1;
      piece.y -= 1;
      ok = !check_collision(&self.grid, &piece, 0, 0);
    }
    if ok {
      self.current_piece = Some(piece);
    }
  }

  fn put_piece_in_grid(&mut self) {
    let piece = self.current_piece.as_ref().unwrap();
    for (i_v_y, line) in piece.cases.iter().enumerate() {
      let i_y = piece.y as usize + i_v_y;
      for (i_v_x, &case) in line.iter().enumerate() {
        if case != Case::Empty {
          let i_x = piece.x as usize + i_v_x;
          self.grid[i_x][i_y] = case;
        }
      }
    }
  }

  fn remove_complete_lines(&mut self) -> u32 {
    let mut line_removed: u32 = 0;
    for y in 0..GRID_HEIGHT {
      let mut all_in_line = true;
      for x in 0..GRID_WIDTH {
        all_in_line &= self.grid[x][y] != Case::Empty;
      }
      if all_in_line {
        line_removed += 1;
        let mut y_to_move = (y -1) as i32;
        while y_to_move >= 0 {
          for x in 0..GRID_WIDTH {
            self.grid[x][y_to_move as usize + 1] = self.grid[x][y_to_move as usize];
          }
          y_to_move -= 1;
        }
      }
    }

    return line_removed;
  }

  fn compute_score(&mut self, line_removed: u32) {
    let factor = match line_removed {
      1 => 40,
      2 => 100,
      3 => 300,
      4 => 1200,
      _ => 0,
    };
    self.score += factor * (self.level as i64);
    println!("Score: {}", self.score);
  }

  fn increase_level(&mut self) {
    if self.line_removed > self.level * 5 {
      self.level += 1;
      self.move_speed = drop_speed(self.level);
      println!("Level: {}", self.level);
      println!("Speed: {:?}", self.move_speed);
    }
  }

  fn generate_piece(&mut self, delta: Duration) -> bool {
    if self.current_piece.is_some() {
      return true;
    }

    self.timer_piece_generation += delta;
    if self.timer_piece_generation > self.move_speed {
      self.timer_piece_generation = Duration::from_secs(0);
      let case = rand::random();
      let piece = create_piece(case);
      let fit_in_grid = !check_collision(&self.grid, &piece, 0, 0);
      self.current_piece = Some(piece);
      return fit_in_grid;
    }
    return true;
  }

  fn piece_move_horizontally(&mut self, dx: i32) {
    if self.current_piece.is_none() {
      return;
    }

    let piece = self.current_piece.as_mut().unwrap();
    if !check_collision(&self.grid, piece, dx, 0) {
      piece.x += dx;
    }
  }

  fn piece_drop(&mut self) {
    if self.current_piece.is_none() {
      return;
    }

    let piece = self.current_piece.as_mut().unwrap();
    while !check_collision(&self.grid, piece, 0, 1) {
      piece.y += 1;
    }
  }

  fn piece_move_down(&mut self, delta: Duration) -> bool {
    if self.current_piece.is_none() {
      return false;
    }

    let dy: i32 = 1;
    let piece = self.current_piece.as_ref().unwrap();
    let should_move = piece.last_move + delta > self.move_speed;
    let can_move = should_move && !check_collision(&self.grid, piece, 0, dy);

    if should_move && !can_move {
      self.put_piece_in_grid();
      self.current_piece = None;
    } else if should_move && can_move {
      let piece = self.current_piece.as_mut().unwrap();
      piece.y += dy;
      piece.last_move = Duration::from_secs(0);
    } else {
      let piece = self.current_piece.as_mut().unwrap();
      piece.last_move += delta;
    }
    return should_move && !can_move;
  }

  fn draw_grid(&mut self, ctx: &mut Context) -> GameResult {
    let gridmesh_builder = &mut graphics::MeshBuilder::new();
    gridmesh_builder.rectangle(
      graphics::DrawMode::stroke(1.0),
      graphics::Rect::new(0.0, 0.0, self.grid_rect.w, self.grid_rect.h),
      graphics::WHITE,
    );
    for i_y in 1..GRID_HEIGHT {
      let y = (i_y as f32) * (CASE_SIZE + CASE_BORDER * 2.0);
      gridmesh_builder.line(
        &[na::Point2::new(0.0, y), na::Point2::new(self.grid_rect.w, y)],
        1.0,
        graphics::WHITE
      )?;
    }
    for i_x in 1..GRID_WIDTH {
      let x = (i_x as f32) * (CASE_SIZE + CASE_BORDER * 2.0);
      gridmesh_builder.line(
        &[na::Point2::new(x, 0.0), na::Point2::new(x, self.grid_rect.h)],
        1.0,
        graphics::WHITE
      )?;
    }
    let grid_mesh = gridmesh_builder.build(ctx)?;

    graphics::draw(ctx, &grid_mesh, (na::Point2::new(self.grid_rect.x, self.grid_rect.y),))?;

    Ok(())
  }

  fn draw_case(&mut self, ctx: &mut Context) -> GameResult {
    for i_x in 0..GRID_WIDTH {
      let x = (i_x as f32) * (CASE_SIZE + CASE_BORDER * 2.0) + CASE_BORDER;
      for i_y in 0..GRID_HEIGHT {
        let case = self.grid[i_x][i_y];
        if case != Case::Empty {
          let y = (i_y as f32) * (CASE_SIZE + CASE_BORDER * 2.0) + CASE_BORDER;
          let mesh_case = graphics::Mesh::new_rectangle(
            ctx, 
            graphics::DrawMode::fill(),
            graphics::Rect::new(x, y, CASE_SIZE as f32, CASE_SIZE as f32),
            case_color(case),
          )?;
          graphics::draw(ctx, &mesh_case, (na::Point2::new(self.grid_rect.x, self.grid_rect.y),))?;
        }
      }
    }

    Ok(())
  }

  fn draw_current_piece(&mut self, ctx: &mut Context) -> GameResult {
    match &self.current_piece {
      Some (piece) => {
        for (i_v_y, line) in piece.cases.iter().enumerate() {
          let i_y = piece.y + i_v_y as i32;
          let y = (i_y as f32) * (CASE_SIZE + CASE_BORDER * 2.0) + CASE_BORDER;
          for (i_v_x, &case) in line.iter().enumerate() {
            if case != Case::Empty {
              let i_x = piece.x + i_v_x as i32;
              let x = (i_x as f32) * (CASE_SIZE + CASE_BORDER * 2.0) + CASE_BORDER;
              let mesh_case = graphics::Mesh::new_rectangle(
                ctx, 
                graphics::DrawMode::fill(),
                graphics::Rect::new(x, y, CASE_SIZE, CASE_SIZE),
                case_color(case),
              )?;
              graphics::draw(ctx, &mesh_case, (na::Point2::new(self.grid_rect.x, self.grid_rect.y),))?;
            }
          }
        }
      },
      None => {},
    };

    Ok(())
  }
}

impl event::EventHandler for MainState {
  fn update(&mut self, ctx: &mut Context) -> GameResult {
    let delta = timer::delta(ctx);

    let end = !self.generate_piece(delta);
    let piece_is_done = !end && self.piece_move_down(delta);
    
    if piece_is_done {
      let line_removed = self.remove_complete_lines();
      if line_removed > 0 {
        self.compute_score(line_removed);
        self.line_removed += line_removed;
        self.increase_level();
      }
    }

    if end {
      self.grid = [[Case::Empty; GRID_HEIGHT]; GRID_WIDTH];
      self.current_piece = None;
      self.move_speed = drop_speed(1);
      self.timer_piece_generation = Duration::from_secs(0);
      self.level = 1;
      self.score = 0;
      self.line_removed = 0;
    }

    Ok(())
  }

  fn key_down_event(&mut self, _ctx: &mut Context, key: event::KeyCode, _mods: event::KeyMods, _: bool) {
    match key {
      event::KeyCode::Left => self.piece_move_horizontally(-1),
      event::KeyCode::Right => self.piece_move_horizontally(1),
      event::KeyCode::Down => self.piece_drop(),
      event::KeyCode::Up => self.rotate(),
      _ => (),
    }
  }

  fn draw(&mut self, ctx: &mut Context) -> GameResult {
    graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());

    self.draw_grid(ctx)?;
    self.draw_case(ctx)?;
    self.draw_current_piece(ctx)?;

    graphics::present(ctx)?;
    Ok(())
  }
}

pub fn main() -> GameResult {
  let cb = ggez::ContextBuilder::new("Tetris", "ggez")
    .window_mode(
      conf::WindowMode::default()
          .fullscreen_type(conf::FullscreenType::Windowed)
          .resizable(false)
          .dimensions(800.0, 600.0));
  let (ctx, event_loop) = &mut cb.build()?;
  let state = &mut MainState::new()?;
  event::run(ctx, event_loop, state)
}
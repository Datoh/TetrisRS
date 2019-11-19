use std::time::Duration;

use ggez;
use ggez::conf;
use ggez::event;
use ggez::graphics;
use ggez::nalgebra as na;
use ggez::timer;
use ggez::{Context, GameResult};

use rand::{ distributions::{Distribution, Standard}, Rng};

const INITIAL_SPEED_MOVE: Duration = Duration::from_secs(1);

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
      vec![Case::Cyan],
      vec![Case::Cyan],
      vec![Case::Cyan],
      vec![Case::Cyan],
    ],
    _ => panic!("Unknow case type"),
  };
}

struct Piece {
  x: i32,
  y: i32,
  last_move: Duration,
  cases: Vec<Vec<Case>>,
}

impl Piece {
  fn width(&self) -> i32 {
    return self.cases[0].len() as i32;
  }
  fn height(&self) -> i32 {
    return self.cases.len() as i32;
  }
}

fn generate_piece(case: Case) -> Piece {
  let cases = piece_cases(case);
  return Piece { x: ((GRID_WIDTH - cases[0].len()) / 2) as i32, y: 0, last_move: Duration::from_secs(0), cases: cases };
}

struct MainState {
  grid: [[Case; GRID_HEIGHT]; GRID_WIDTH],
  grid_rect: graphics::Rect,
  current_piece: Option<Piece>,
  move_speed: Duration,
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
      move_speed: INITIAL_SPEED_MOVE,
    };
    Ok(s)
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

  fn piece_check_collision(&self, dx: i32, dy: i32) -> bool {
    let piece = self.current_piece.as_ref().unwrap();
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
          if self.grid[i_x][i_y] != Case::Empty {
            return true;
          }
        }
      }
    }

    return false;
  }

  fn piece_move_horizontally(&mut self, dx: i32) {
    if self.current_piece.is_none() {
      return;
    }

    if !self.piece_check_collision(dx, 0) {
      let piece = self.current_piece.as_mut().unwrap();
      piece.x += dx;
    }
  }

  fn piece_drop(&mut self) {
    if self.current_piece.is_none() {
      return;
    }
    let dy = 1;
    while !self.piece_check_collision(0, dy) {
      let piece = self.current_piece.as_mut().unwrap();
      piece.y += dy;
    }
  }

  fn piece_move_down(&mut self, delta: Duration) {
    if self.current_piece.is_none() {
      return;
    }

    let dy: i32 = 1;
    let piece = self.current_piece.as_ref().unwrap();
    let should_move = piece.last_move + delta > self.move_speed;
    let can_move = should_move && !self.piece_check_collision(0, dy);

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
    self.piece_move_down(delta);
    Ok(())
  }

  fn key_down_event(&mut self, _ctx: &mut Context, key: event::KeyCode, _mods: event::KeyMods, _: bool) {
    match key {
        event::KeyCode::R => {
          let case = rand::random();
          self.current_piece = Some(generate_piece(case));
        }
        event::KeyCode::Left => {
          self.piece_move_horizontally(-1);
        }
        event::KeyCode::Right => {
          self.piece_move_horizontally(1);
        }
        event::KeyCode::Down => {
          self.piece_drop();
        }
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
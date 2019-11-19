use ggez;
use ggez::conf;
use ggez::event;
use ggez::graphics;
use ggez::nalgebra as na;
use ggez::{Context, GameResult};

use rand::Rng;

const GRID_WIDTH: usize = 10;
const GRID_HEIGHT: usize = 20;

type Case = u8;

const CASE_NONE:        Case = 0;
const CASE_RED:         Case = 1;
const CASE_GREEN:       Case = 2;
const CASE_BLUE:        Case = 3;
const CASE_YELLOW:      Case = 4;
const CASE_DARK_YELLOW: Case = 5;
const CASE_PURPLE:      Case = 6;
const CASE_CYAN:        Case = 7;

const CASE_SIZE:   u32 = 20;
const CASE_BORDER: u32 = 2;

fn case_color(case: Case) -> graphics::Color {
  return match case {
    CASE_RED => graphics::Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 },
    CASE_GREEN => graphics::Color { r: 0.0, g: 1.0, b: 0.0, a: 1.0 },
    CASE_BLUE => graphics::Color { r: 0.0, g: 0.0, b: 1.0, a: 1.0 },
    CASE_YELLOW => graphics::Color { r: 1.0, g: 1.0, b: 0.0, a: 1.0 },
    CASE_DARK_YELLOW => graphics::Color { r: 1.0, g: 0.85, b: 0.0, a: 1.0 },
    CASE_PURPLE => graphics::Color { r: 0.5, g: 0.0, b: 0.5, a: 1.0 },
    CASE_CYAN => graphics::Color { r: 0.0, g: 1.0, b: 1.0, a: 1.0 },
    _ => panic!("Unknow case type {}", case),
  };
}

fn piece_cases(case: Case) -> Vec<Vec<Case>> {
  return match case {
    CASE_RED => vec![
      vec![CASE_RED, CASE_RED, CASE_NONE], 
      vec![CASE_NONE, CASE_RED, CASE_RED],
    ],
    CASE_GREEN => vec![
      vec![CASE_NONE, CASE_GREEN, CASE_GREEN],
      vec![CASE_GREEN, CASE_GREEN, CASE_NONE], 
    ],
    CASE_BLUE => vec![
      vec![CASE_BLUE, CASE_NONE, CASE_NONE],
      vec![CASE_BLUE, CASE_BLUE, CASE_BLUE], 
    ],
    CASE_YELLOW => vec![
      vec![CASE_NONE, CASE_NONE, CASE_YELLOW],
      vec![CASE_YELLOW, CASE_YELLOW, CASE_YELLOW], 
    ],
    CASE_DARK_YELLOW => vec![
      vec![CASE_DARK_YELLOW, CASE_DARK_YELLOW],
      vec![CASE_DARK_YELLOW, CASE_DARK_YELLOW], 
    ],
    CASE_PURPLE => vec![
      vec![CASE_NONE, CASE_PURPLE, CASE_NONE],
      vec![CASE_PURPLE, CASE_PURPLE, CASE_PURPLE], 
    ],
    CASE_CYAN => vec![
      vec![CASE_CYAN],
      vec![CASE_CYAN],
      vec![CASE_CYAN],
      vec![CASE_CYAN],
    ],
    _ => panic!("Unknow case type {}", case),
  };
}

struct Piece {
  x: u32,
  y: u32,
  cases: Vec<Vec<Case>>,
}

fn generate_piece(case: Case) -> Piece {
  let cases = piece_cases(case);
  return Piece { x: ((GRID_WIDTH - cases[0].len()) / 2) as u32, y: 0, cases: cases };
}

struct MainState {
  grid: [[Case; GRID_HEIGHT]; GRID_WIDTH],
  current_piece: Option<Piece>,
}

impl MainState {
  fn new() -> GameResult<MainState> {
    let s = MainState { grid: [[CASE_NONE; GRID_HEIGHT]; GRID_WIDTH], current_piece: None };
    Ok(s)
  }

  fn draw_grid(&mut self, ctx: &mut Context) -> GameResult {
    let width = ((GRID_WIDTH as u32) * (CASE_SIZE + 2 * CASE_BORDER)) as f32;
    let height = ((GRID_HEIGHT as u32) * (CASE_SIZE + 2 * CASE_BORDER)) as f32;
    let left = (800.0 - width as f32) / 2.0;
    let top = (600.0 - height as f32) / 2.0;

    let gridmesh_builder = &mut graphics::MeshBuilder::new();
    gridmesh_builder.rectangle(
      graphics::DrawMode::stroke(1.0),
      graphics::Rect::new(0.0, 0.0, width, height),
      graphics::WHITE,
    );
    for i_y in 1..GRID_HEIGHT {
      let y = (i_y as u32 * (CASE_SIZE + CASE_BORDER * 2)) as f32;
      gridmesh_builder.line(
        &[na::Point2::new(0.0, y), na::Point2::new(width, y)],
        1.0,
        graphics::WHITE
      )?;
    }
    for i_x in 1..GRID_WIDTH {
      let x = (i_x as u32 * (CASE_SIZE + CASE_BORDER * 2)) as f32;
      gridmesh_builder.line(
        &[na::Point2::new(x, 0.0), na::Point2::new(x, height)],
        1.0,
        graphics::WHITE
      )?;
    }
    let grid_mesh = gridmesh_builder.build(ctx)?;

    graphics::draw(ctx, &grid_mesh, (na::Point2::new(left, top),))?;

    Ok(())
  }

  fn draw_case(&mut self, ctx: &mut Context) -> GameResult {
    let width = ((GRID_WIDTH as u32) * (CASE_SIZE + 2 * CASE_BORDER)) as f32;
    let height = ((GRID_HEIGHT as u32) * (CASE_SIZE + 2 * CASE_BORDER)) as f32;
    let left = (800.0 - width as f32) / 2.0;
    let top = (600.0 - height as f32) / 2.0;

    for i_x in 0..GRID_WIDTH {
      let x = (i_x as u32 * (CASE_SIZE + CASE_BORDER * 2) + CASE_BORDER) as f32;
      for i_y in 0..GRID_HEIGHT {
        let case = self.grid[i_x][i_y];
        if case != CASE_NONE {
          let y = (i_y as u32 * (CASE_SIZE + CASE_BORDER * 2) + CASE_BORDER) as f32;
          let mesh_case = graphics::Mesh::new_rectangle(
            ctx, 
            graphics::DrawMode::fill(),
            graphics::Rect::new(x, y, CASE_SIZE as f32, CASE_SIZE as f32),
            case_color(case),
          )?;
          graphics::draw(ctx, &mesh_case, (na::Point2::new(left, top),))?;
        }
      }
    }

    Ok(())
  }

  fn draw_current_piece(&mut self, ctx: &mut Context) -> GameResult {
    match &self.current_piece {
      Some(piece) => {
        let width = ((GRID_WIDTH as u32) * (CASE_SIZE + 2 * CASE_BORDER)) as f32;
        let height = ((GRID_HEIGHT as u32) * (CASE_SIZE + 2 * CASE_BORDER)) as f32;
        let left = (800.0 - width as f32) / 2.0;
        let top = (600.0 - height as f32) / 2.0;

        for (i_v_y, line) in piece.cases.iter().enumerate() {
          let i_y = piece.y + i_v_y as u32;
          let y = (i_y * (CASE_SIZE + CASE_BORDER * 2) + CASE_BORDER) as f32;
          for (i_v_x, case) in line.iter().enumerate() {
            if *case != CASE_NONE {
              let i_x = piece.x + i_v_x as u32;
              let x = (i_x * (CASE_SIZE + CASE_BORDER * 2) + CASE_BORDER) as f32;
              let mesh_case = graphics::Mesh::new_rectangle(
                ctx, 
                graphics::DrawMode::fill(),
                graphics::Rect::new(x, y, CASE_SIZE as f32, CASE_SIZE as f32),
                case_color(*case),
              )?;
              graphics::draw(ctx, &mesh_case, (na::Point2::new(left, top),))?;
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
  fn update(&mut self, _ctx: &mut Context) -> GameResult {
    Ok(())
  }

  fn key_down_event(&mut self, _ctx: &mut Context, key: event::KeyCode, _mods: event::KeyMods, _: bool) {
    match key {
        event::KeyCode::R => {
          let mut r = rand::thread_rng();
          let case = r.gen_range(CASE_RED, CASE_CYAN + 1);
          self.current_piece = Some(generate_piece(case));
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
use core::fmt;
use std::collections::HashMap;

use board::{Board, BoardUnionFind, BoardVec};

use crate::board::BoardUnionId;

pub mod ai;
pub mod board;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Field {
  Unknown,
  Snake,
  SnakeEnd,
  Empty,
}

impl Field {
  pub fn is_snake(self) -> bool {
    matches!(self, Field::Snake | Field::SnakeEnd)
  }

  pub fn is_empty(self) -> bool {
    matches!(self, Field::Empty)
  }

  pub fn or_snake(self) -> Self {
    if self.is_empty() {
      Self::Snake
    } else {
      self
    }
  }

  pub fn max_snake_neighbours(self) -> usize {
    match self {
      Field::Snake => 2,
      Field::SnakeEnd => 1,
      Field::Empty | Field::Unknown => 4,
    }
  }
}

impl fmt::Display for Field {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Field::Unknown => write!(f, " "),
      Field::Snake => write!(f, "+"),
      Field::SnakeEnd => write!(f, "X"),
      Field::Empty => write!(f, "·"),
    }
  }
}

pub type GameBoard = Board<Field>;
pub type Segment = usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnakeConnectedness {
  Unconnected,
  Connected,
  Distributed,
}

#[derive(Clone)]
pub struct State {
  board: GameBoard,
  unions: BoardUnionFind<u32>,
  snake_ends: Vec<BoardVec>,
  unknowns: u32,
  snake_count: usize,
}

impl State {
  pub fn new_empty(width: u32, height: u32) -> Self {
    let board = GameBoard::new(width, height, Field::Unknown);
    let unions = BoardUnionFind::new(width, height);
    for pos in board.positions() {
      unions[pos].set_data(board.get_pos_around_4(pos).count() as u32);
    }

    Self {
      board,
      unions,
      snake_ends: Vec::new(),
      unknowns: width * height,
      snake_count: 0,
    }
  }

  pub fn new_rand(width: u32, height: u32) -> Self {
    let mut rng = rand::thread_rng();

    let size = BoardVec::new(width as i32, height as i32);
    let a = size.rand(&mut rng);

    loop {
      let b = size.rand(&mut rng);

      if a.dist(b) >= 2 {
        return Self::new(width, height, a, b);
      }
    }
  }

  pub fn new(width: u32, height: u32, a: BoardVec, b: BoardVec) -> Self {
    assert!(a.dist(b) >= 2);

    let mut state = Self::new_empty(width, height);

    state.set(a, Field::SnakeEnd);
    state.set(b, Field::SnakeEnd);

    state
  }

  pub fn width(&self) -> u32 {
    self.board.width
  }

  pub fn height(&self) -> u32 {
    self.board.height
  }

  pub fn field(&self, pos: BoardVec) -> Field {
    self.board[pos]
  }

  pub fn pos_around(&self, pos: BoardVec) -> impl Iterator<Item = BoardVec> + '_ {
    self.board.get_pos_around_4(pos)
  }

  pub fn set(&mut self, pos: BoardVec, value: Field) {
    let field = self.field(pos);

    assert_eq!(field, Field::Unknown);

    self.unknowns -= 1;

    match value {
      Field::Unknown => panic!("Cannot set Field::Unknown"),
      Field::Snake | Field::SnakeEnd => {
        assert!(self.snake_allowed(pos));
        self.board[pos] = value;
        self.snake_count += 1;

        for p in self.board.get_pos_around_4(pos) {
          let u = &self.unions[p];
          u.set_data(u.data() - 1);
          if self.field(p).is_snake() {
            let (merged, _) = self.unions.merge(pos, p);
            debug_assert!(merged);
          }
        }

        if value == Field::SnakeEnd {
          self.snake_ends.push(pos);
        }
      }
      Field::Empty => {
        assert!(self.empty_allowed(pos));
        self.board[pos] = value;

        for p in self.board.get_pos_around_4(pos) {
          let u = &self.unions[p];
          u.set_data(u.data() - 1);

          if self.field(p).is_empty() {
            self.unions.merge(pos, p);
          }
        }
      }
    }
  }

  pub fn is_dangling_snake(&self, pos: BoardVec) -> bool {
    let field = self.field(pos);
    field.is_snake() && self.snakes_around(pos) < field.max_snake_neighbours()
  }

  pub fn snakes_around(&self, pos: BoardVec) -> usize {
    self.board.get_around_4(pos).filter(|f| f.is_snake()).count()
  }

  pub fn unknown_around(&self, pos: BoardVec) -> usize {
    self.board.get_around_4(pos).filter(|&&f| f == Field::Unknown).count()
  }

  pub fn snake_allowed(&self, pos: BoardVec) -> bool {
    if self.field(pos) != Field::Unknown {
      return false;
    }

    let snakes_around = self.snakes_around(pos);
    if snakes_around == 0 {
      return true;
    }

    if snakes_around > 2 {
      return false;
    }

    let mut first_seg: Option<BoardUnionId> = None;
    let mut empty_clusters: HashMap<BoardUnionId, u32> = HashMap::new();

    for p in self.pos_around(pos) {
      let field = self.field(p);
      if field.is_snake() {
        let seg = self.unions[p].id();
        let segs_are_same = first_seg == Some(seg);
        first_seg = Some(seg);
        if segs_are_same || !self.is_dangling_snake(p) {
          return false;
        }
      } else if field.is_empty() {
        let u = &self.unions[p];
        let cluster = empty_clusters.entry(u.id());
        let cluster = cluster.or_insert_with(|| u.data());
        if *cluster <= 1 && u.size() <= 4 {
          //println!("{:?}", pos);
          //println!("{:?}", self);
          return false;
        }
        *cluster -= 1;
      }
    }

    true
  }

  pub fn empty_allowed(&self, pos: BoardVec) -> bool {
    if self.field(pos) != Field::Unknown {
      return false;
    }

    let mut cluster_count = 1;
    let mut empty_clusters: HashMap<BoardUnionId, u32> = HashMap::new();

    for p in self.pos_around(pos) {
      let field = self.field(p);
      if field.is_empty() {
        let u = &self.unions[p];
        let c = empty_clusters.entry(u.id()).or_insert_with(|| {
          cluster_count += u.size();
          u.data()
        });
        *c -= 1;
      } else if field.is_snake() {
        let snakes_around = self.snakes_around(p);
        let unknown_around = self.unknown_around(p);
        debug_assert!(unknown_around > 0);

        if unknown_around <= field.max_snake_neighbours() - snakes_around {
          return false;
        }
      }
    }

    let will_not_be_closed = self.unions[pos].data() >= 1 || empty_clusters.values().any(|&c| c > 0);
    cluster_count == 5 || cluster_count <= 5 && will_not_be_closed
  }

  pub fn is_snake_connected(&self) -> SnakeConnectedness {
    let a = self.snake_ends.get(0);
    let b = self.snake_ends.get(1);

    if let (Some(&a), Some(&b)) = (a, b) {
      let a = &self.unions[a];
      if a == &self.unions[b] {
        return if a.size() == self.snake_count {
          SnakeConnectedness::Connected
        } else {
          SnakeConnectedness::Distributed
        };
      }
    }
    SnakeConnectedness::Unconnected
  }

  pub fn unknowns(&self) -> u32 {
    self.unknowns
  }
}

impl fmt::Debug for State {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    writeln!(f, "x{}x", "-".repeat(self.width() as usize))?;
    for y in 0..self.height() {
      write!(f, "|")?;
      for x in 0..self.width() {
        let pos = BoardVec::new(x as i32, y as i32);

        if self.field(pos) == Field::Unknown {
          write!(f, "{}", self.unions[pos].data())?;
        } else {
          write!(f, "{}", self.field(pos))?;
        }
      }
      writeln!(f, "|")?;
    }
    writeln!(f, "x{}x", "-".repeat(self.width() as usize))?;

    Ok(())
  }
}
/*
type Op = (BoardVec, Field);

fn interesting_fields(state: &State) -> impl Iterator<Item = BoardVec> + '_ {
  state.board.positions().filter(|&pos| {
    state.field(pos) == Field::Unknown && state.pos_around(pos).any(|p| state.field(p) != Field::Unknown)
  })
}*/

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SolveResult {
  Contradiction,
  MaxDepth,
}

pub fn solve(
  mut state: State,
  rest_depth: usize,
  fails: &mut Vec<(BoardVec, State)>,
) -> Result<SolveResult, State> {
  //println!("{:?}", state);
  loop {
    let mut changed = false;
    for pos in state.board.positions() {
      if state.field(pos) != Field::Unknown {
        continue;
      }

      let snake_allowed = state.snake_allowed(pos);
      let empty_allowed = state.empty_allowed(pos);

      if !snake_allowed && !empty_allowed {
        fails.push((pos, state));
        return Ok(SolveResult::Contradiction);
      } else if !snake_allowed {
        state.set(pos, Field::Empty);
        changed = true;
      } else if !empty_allowed {
        state.set(pos, Field::Snake);
        changed = true;
      }
    }
    if !changed {
      break;
    }
  }

  if rest_depth == 0 {
    panic!("oh no");
    //return Ok(SolveResult::MaxDepth);
  }

  for pos in state.board.positions() {
    if state.field(pos) == Field::Unknown {
      let snake_result = {
        let mut s = state.clone();
        s.set(pos, Field::Snake);
        solve(s, rest_depth - 1, fails)?
      };
      let empty_result = {
        let mut s = state.clone();
        s.set(pos, Field::Empty);
        solve(s, rest_depth - 1, fails)?
      };

      match (snake_result, empty_result) {
        (SolveResult::Contradiction, SolveResult::Contradiction) => return Ok(SolveResult::Contradiction),
        (SolveResult::Contradiction, SolveResult::MaxDepth) => state.set(pos, Field::Empty),
        (SolveResult::MaxDepth, SolveResult::Contradiction) => state.set(pos, Field::Snake),
        (SolveResult::MaxDepth, SolveResult::MaxDepth) => panic!("blbu"),
      }
    }
  }

  let connected = state.is_snake_connected();

  //if connected == SnakeConnectedness::Distributed {
  //  return Ok(SolveResult::Contradiction);
  //}

  if state.unknowns == 0 {
    return if connected == SnakeConnectedness::Connected {
      Err(state)
    } else {
      //println!("{:?}", state);
      fails.push((BoardVec::new(-1, -1), state));
      Ok(SolveResult::Contradiction)
    };
  }

  solve(state, rest_depth + 1, fails)
}

use core::fmt;
use std::cell::Cell;
use std::collections::HashMap;

use board::{Board, BoardVec};
use rand::seq::{IteratorRandom, SliceRandom};
use rand::{thread_rng, Rng};

pub mod ai;
pub mod board;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Field {
  Snake,
  SnakeEnd,
  Empty,
}

impl Field {
  pub fn is_snake(self) -> bool {
    !self.is_blank()
  }

  pub fn is_blank(self) -> bool {
    matches!(self, Field::Empty)
  }

  pub fn or_snake(self) -> Self {
    if self.is_blank() {
      Self::Snake
    } else {
      self
    }
  }

  pub fn max_snake_neighbours(self) -> usize {
    match self {
      Field::Snake => 2,
      Field::SnakeEnd => 1,
      Field::Empty => 4,
    }
  }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum ConnectPriority {
  None,
  Horizontal,
  Vertical,
}

impl ConnectPriority {
  fn from(a: BoardVec, b: BoardVec) -> Self {
    assert_ne!(a, b);

    if a.x == b.x {
      Self::Horizontal
    } else if a.y == b.y {
      Self::Vertical
    } else {
      panic!("a and b must be either have same x or same y")
    }
  }
}

impl fmt::Display for Field {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Field::Snake => write!(f, "+"),
      Field::SnakeEnd => write!(f, "X"),
      Field::Empty => write!(f, " "),
    }
  }
}

pub type GameBoard = Board<Field>;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Game {
  board: GameBoard,
}

impl Game {
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

    let mut board = GameBoard::new(width, height, Field::Empty);

    board[a] = Field::SnakeEnd;
    board[b] = Field::SnakeEnd;

    let mut game = Self { board };
    assert!(game.connect_snake(a, b, ConnectPriority::None));
    game
  }

  pub fn width(&self) -> u32 {
    self.board.width
  }

  pub fn height(&self) -> u32 {
    self.board.height
  }

  pub fn snake_allowed(&self, pos: BoardVec) -> bool {
    let mut i = 0;
    self
      .board
      .get_pos_around_4(pos)
      .filter(|&p| self.board[p].is_snake())
      .inspect(|_| i += 1)
      .all(|p| self.is_dangling_snake(p))
      && i <= 2
  }

  pub fn is_dangling_snake(&self, pos: BoardVec) -> bool {
    let field = self.board[pos];
    field.is_snake()
      && self.board.get_around_4(pos).filter(|f| f.is_snake()).count() < field.max_snake_neighbours()
  }

  pub fn evolve(&mut self) -> (bool, bool) {
    let rng = &mut thread_rng();
    let dangling_snakes: Vec<_> = self
      .board
      .positions()
      .filter(|&pos| self.is_dangling_snake(pos))
      .collect();

    for &pos in dangling_snakes.iter() {
      if self.is_dangling_snake(pos) {
        let target = self
          .board
          .get_pos_around_4(pos)
          .filter(|&p| self.board[p].is_blank())
          .filter(|&p| self.snake_allowed(p))
          .choose(rng);
        if let Some(target) = target {
          self.board[target] = Field::Snake;
        }
      }
    }

    let is_complete_old = !self.board.positions().any(|pos| self.is_dangling_snake(pos));
    let is_complete = self.is_complete();
    if !dangling_snakes.is_empty() && (is_complete || is_complete_old) || rng.gen_range(0..3) == 0 {
      return (is_complete, is_complete_old);
    }

    let snakes: Vec<_> = self
      .board
      .positions()
      .filter(|&pos| self.board[pos] == Field::Snake)
      .collect();

    let delete = rng.gen_range(0..2.max(snakes.len() / 10).min(5));
    for &snake in snakes.choose_multiple(rng, delete) {
      self.board[snake] = Field::Empty;
    }
    if delete == 0 || snakes.is_empty() {
      (is_complete, is_complete_old)
    } else {
      (false, false)
    }
  }

  fn is_complete(&self) -> bool {
    let mut snakes = 0;
    let mut start = BoardVec::new(0, 0);
    for pos in self.board.positions() {
      match self.board[pos] {
        Field::Snake => snakes += 1,
        Field::SnakeEnd => start = pos,
        Field::Empty => {}
      }
    }

    let snakes = snakes;
    let mut prev = start;
    let mut cur = start;
    let mut found = 0;

    'next: loop {
      for next in self.board.get_pos_around_4(cur) {
        if next != prev {
          let field = self.board[next];
          if field == Field::SnakeEnd {
            return found == snakes;
          }
          if field == Field::Snake {
            prev = cur;
            cur = next;
            found += 1;
            continue 'next;
          }
        }
      }
      return false;
    }
  }

  fn connect_snake(&mut self, a: BoardVec, b: BoardVec, prio: ConnectPriority) -> bool {
    fn vertical(game: &mut Game, pos: &mut BoardVec, b: BoardVec) -> bool {
      let dir: i32 = (b.y - pos.y).signum();
      while pos.y != b.y {
        if game.snake_allowed(*pos) {
          game.board[*pos] = Field::Snake;
          pos.y += dir;
        } else {
          return false;
        }
      }
      true
    }

    fn horizontal(game: &mut Game, pos: &mut BoardVec, b: BoardVec) -> bool {
      let dir: i32 = (b.x - pos.x).signum();
      while pos.x != b.x {
        if game.snake_allowed(*pos) {
          game.board[*pos] = Field::Snake;
          pos.x += dir;
        } else {
          return false;
        }
      }
      true
    }

    let a_f = self.board[a].or_snake();
    let b_f = self.board[b].or_snake();

    let mut pos = a;
    let res = if prio == ConnectPriority::Horizontal {
      horizontal(self, &mut pos, b) && vertical(self, &mut pos, b)
    } else {
      vertical(self, &mut pos, b) && horizontal(self, &mut pos, b)
    };

    self.board[a] = a_f;
    self.board[b] = b_f;
    res
  }

  pub fn shake_snake(&mut self) {
    let rng = &mut thread_rng();
    let snakes: Vec<_> = self
      .board
      .enumerate()
      .flat_map(|(pos, &f)| (f == Field::Snake).then_some(pos))
      .collect();

    let mut i = 0;
    'blub: loop {
      i += 1;
      if i == 100 {
        return;
      }

      let snake = *snakes.choose(rng).unwrap();
      let snake_neighbours: Vec<_> = snake
        .neighbours_4()
        .filter(|&p| self.board.get(p).filter(|f| f.is_snake()).is_some())
        .collect();
      debug_assert!(snake_neighbours.len() == 2);

      let target = self
        .board
        .get_pos_around_8(snake)
        .filter(|&p| self.board[p].is_blank())
        .collect::<Vec<_>>()
        .choose(rng)
        .cloned();
      let target = if let Some(target) = target {
        target
      } else {
        continue;
      };

      let mut copy = self.clone();
      copy.board[snake] = Field::Empty;

      //println!("Try {:?} -> {:?} ({:?})", snake, target, snake_neighbours);

      for p in snake_neighbours {
        if !copy.connect_snake(p, target, ConnectPriority::from(p, snake)) {
          continue 'blub;
        }
      }

      *self = copy;
      return;
    }
  }
}

impl fmt::Debug for Game {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    writeln!(f, "+{}+", "-".repeat(self.width() as usize))?;
    for y in 0..self.height() {
      write!(f, "|")?;
      for x in 0..self.width() {
        let pos = BoardVec::new(x as i32, y as i32);
        write!(f, "{}", self.board[pos])?;
      }
      writeln!(f, "|")?;
    }
    writeln!(f, "+{}+", "-".repeat(self.width() as usize))?;

    Ok(())
  }
}

/*
pub struct GameSetupBuilder {
  mines: Board<bool>,
  protected: Board<bool>,
  rng: Box<dyn RngCore>,
}

impl GameSetupBuilder {
  pub fn new(width: u32, height: u32) -> Self {
    Self {
      mines: Board::new(width, height, false),
      protected: Board::new(width, height, false),
      rng: Box::new(rand::thread_rng()),
    }
  }

  pub fn has_mine(&self, pos: BoardVec) -> bool {
    self.mines[pos]
  }

  pub fn set_mine(&mut self, pos: BoardVec) {
    assert!(!self.is_protected(pos));
    self.mines[pos] = true;
  }

  pub fn is_protected(&self, pos: BoardVec) -> bool {
    self.protected[pos]
  }

  pub fn protect(&mut self, pos: BoardVec) {
    self.mines[pos] = false;
    self.protected[pos] = true;
  }

  pub fn protect_all(&mut self, all: impl IntoIterator<Item = BoardVec>) {
    for pos in all {
      self.protect(pos);
    }
  }

  pub fn add_random_mines(&mut self, mut mines: u32) -> bool {
    let mut possible_positions: Vec<_> = self.mines.positions().collect();
    possible_positions.shuffle(&mut self.rng);

    while let Some(pos) = possible_positions.pop() {
      if mines == 0 {
        return true;
      }

      if self.is_protected(pos) || self.has_mine(pos) {
        continue;
      }

      self.set_mine(pos);
      mines -= 1;
    }

    false
  }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Game {
  setup: GameSetup,
  view: ViewBoard,
}

impl Game {
  pub fn setup(&self) -> &GameSetup {
    &self.setup
  }

  pub fn board(&self) -> &GameBoard {
    &self.setup.board
  }

  pub fn width(&self) -> u32 {
    self.board().width
  }

  pub fn height(&self) -> u32 {
    self.board().height
  }

  pub fn is_visible(&self, pos: BoardVec) -> bool {
    self.view[pos]
  }

  pub fn view(&self, pos: BoardVec) -> Option<Field> {
    if self.is_visible(pos) {
      self.board().get(pos).copied()
    } else {
      None
    }
  }

  pub fn open(&mut self, pos: BoardVec) -> bool {
    assert!(!self.view[pos]);
    if self.board()[pos].is_mine() {
      return false;
    }

    let mut explorer = BoardExplorer::from(self.board());
    explorer.enqueue(pos);

    while let Some(pos) = explorer.pop() {
      self.view[pos] = true;
      if self.board()[pos].is_blank() {
        explorer.enqueue_all(pos.neighbours());
      }
    }

    true
  }
}

impl From<GameSetup> for Game {
  fn from(setup: GameSetup) -> Self {
    Self {
      view: ViewBoard::new(setup.width(), setup.height(), false),
      setup,
    }
  }
}

impl<B: Borrow<GameSetupBuilder>> From<B> for Game {
  fn from(setup: B) -> Self {
    Self::from(GameSetup::from(setup))
  }
}

impl fmt::Debug for Game {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    for y in 0..self.height() {
      for x in 0..self.width() {
        let pos = BoardVec::new(x as i32, y as i32);
        if self.is_visible(pos) {
          write!(f, "{}", self.board()[pos])?;
        } else {
          write!(f, "░")?;
        }
      }
      writeln!(f)?;
    }

    Ok(())
  }
}


#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub enum FieldView {
  Open,
  Hidden,
  Flagged,
}

impl FieldView {
  pub fn is_open(self) -> bool {
    self == FieldView::Open
  }

  pub fn is_hidden(self) -> bool {
    !self.is_open()
  }

  pub fn is_flagged(self) -> bool {
    self == FieldView::Flagged
  }
}*/

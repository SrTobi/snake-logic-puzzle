use core::fmt;
use std::cell::Cell;
use std::mem;

use board::{Board, BoardUnionFind, BoardVec, BoardExplorer};
use rand::seq::{IteratorRandom, SliceRandom};
use rand::{thread_rng, Rng};

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

#[derive(Clone)]
pub struct GameCreator {
  board: GameBoard,
  snake_ends: (BoardVec, BoardVec),
  unions: BoardUnionFind,
}

impl GameCreator {
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

    let mut creator = Self {
      board: Board::new(width, height, Field::Unknown),
      snake_ends: (a, b),
      unions: BoardUnionFind::new(width, height),
    };

    creator.set_snake(a);
    creator.board[a] = Field::SnakeEnd;

    creator.set_snake(b);
    creator.board[b] = Field::SnakeEnd;

    creator
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

  pub fn set_snake(&mut self, pos: BoardVec) {
    let field = self.field(pos);
  
    debug_assert_eq!(field, Field::Unknown);
    debug_assert!(self.snake_allowed(pos));
  
    self.board[pos] = Field::Snake;

    for p in self.board.get_pos_around_4(pos) {
      if self.field(p).is_snake() {
        let (merged, _) = self.unions.merge(pos, p);
        debug_assert!(merged);
      }
    }
  }

  pub fn set_empty(&mut self, pos: BoardVec) {
    debug_assert_eq!(self.field(pos), Field::Empty);
    self.board[pos] = Field::Snake;
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

    self
      .pos_around(pos)
      .filter(|&p| self.field(p).is_snake())
      .all(move |p| {
        let seg = self.unions[p].id();
        let segs_are_same = first_seg == Some(seg);
        first_seg = Some(seg);
        !segs_are_same && self.is_dangling_snake(p)
      })
  }


  pub fn is_dangling_snake(&self, pos: BoardVec) -> bool {
    let field = self.field(pos);
    field.is_snake() && self.snakes_around(pos) < field.max_snake_neighbours()
  }

  pub fn snakes_around(&self, pos: BoardVec) -> usize {
    self.board.get_around_4(pos).filter(|f| f.is_snake()).count()
  }

  fn is_snake_connected(&self) -> bool {
    let (a, b) = self.snake_ends;

    self.unions[a] == self.unions[b]
  }
}

impl fmt::Debug for GameCreator {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    writeln!(f, "+{}+", "-".repeat(self.width() as usize))?;
    for y in 0..self.height() {
      write!(f, "|")?;
      for x in 0..self.width() {
        let pos = BoardVec::new(x as i32, y as i32);
        if self.field(pos).is_blank() {
          // write!(f, "{}", self.snakes_around[pos])?;
          write!(f, "{}", if self.snake_allowed(pos) { 'O' } else { ' ' })?;
        } else {
          write!(f, "{}", self.field(pos))?;
        }
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

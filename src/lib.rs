#![feature(iter_advance_by)]
#![feature(get_mut_unchecked)]
#![feature(binary_heap_drain_sorted)]

use core::fmt;
use std::collections::HashMap;
use std::hash::Hash;

use board::{Board, BoardUnion, BoardUnionFind, BoardVec};

use crate::board::BoardUnionId;

pub mod ai;
pub mod board;
pub mod list;
pub mod serialize;
pub mod solver;

pub use solver::*;

struct Throwaway;

impl<T> Extend<T> for Throwaway {
  fn extend<I: IntoIterator<Item = T>>(&mut self, _: I) {}
}

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

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub enum EmptyPolicy {
  None,
  Fix(usize),
  Ascending(Vec<bool>, usize),
}

impl EmptyPolicy {
  pub fn new_ascending(width: u32, height: u32) -> Self {
    let upper_field_limit = width * height - 3 /* at least 3 snakes per game */;
    let mut max = 0;
    let mut fields = 0;

    while fields <= upper_field_limit {
      max += 1;
      fields += max;
    }

    //println!("max: {max}");
    Self::Ascending(Vec::new(), (max - 1) as usize)
  }

  pub fn allowed(&self, empty_fields: usize) -> bool {
    match self {
      EmptyPolicy::None => true,
      &EmptyPolicy::Fix(n) => empty_fields == n,
      EmptyPolicy::Ascending(v, _) => {
        self.could_become_allowed(empty_fields) && !v.get(empty_fields - 1).unwrap_or(&false)
      }
    }
  }

  pub fn could_become_allowed(&self, empty_fields: usize) -> bool {
    match self {
      EmptyPolicy::None => true,
      &EmptyPolicy::Fix(n) => empty_fields <= n,
      EmptyPolicy::Ascending(_, max) => empty_fields <= *max,
    }
  }

  pub fn is_still_possible(&self, unenclosed_fields_left: usize) -> bool {
    match self {
      EmptyPolicy::None => true,
      EmptyPolicy::Fix(_) => true,
      EmptyPolicy::Ascending(v, _) => {
        let needed: usize = v
          .iter()
          .enumerate()
          .map(|(i, &taken)| if taken { 0 } else { i + 1 })
          .sum();
        needed <= unenclosed_fields_left
      }
    }
  }

  pub fn notify(&mut self, empty_fields: usize) {
    match self {
      EmptyPolicy::None => (),
      EmptyPolicy::Fix(_) => (),
      EmptyPolicy::Ascending(v, _) => {
        if v.len() < empty_fields {
          v.resize(empty_fields, false);
        }
        assert!(!v[empty_fields - 1]);
        v[empty_fields - 1] = true;
      }
    }
  }
}

#[derive(Clone)]
pub struct State {
  board: GameBoard,
  unions: BoardUnionFind<u32>,
  snake_ends: Vec<BoardVec>,
  unknowns: u32,
  unenclosed_empties: usize,
  snake_count: usize,
  empty_policy: EmptyPolicy,
}

impl State {
  pub fn new_empty(width: u32, height: u32, ep: EmptyPolicy) -> Self {
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
      unenclosed_empties: 0,
      snake_count: 0,
      empty_policy: ep,
    }
  }

  pub fn new_rand(width: u32, height: u32, ep: EmptyPolicy) -> Self {
    let mut rng = rand::thread_rng();

    let size = BoardVec::new(width as i32, height as i32);
    let a = size.rand(&mut rng);

    loop {
      let b = size.rand(&mut rng);

      if a.dist(b) >= 2 {
        return Self::new(width, height, a, b, ep);
      }
    }
  }

  pub fn new(width: u32, height: u32, a: BoardVec, b: BoardVec, ep: EmptyPolicy) -> Self {
    assert!(a.dist(b) >= 2);

    let mut state = Self::new_empty(width, height, ep);

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
          } else if self.field(p).is_empty() && u.data() == 0 {
            //println!("{:?} -> {:?} ({:?})", pos, p, u);
            //println!("{:?}", self);
            self.empty_policy.notify(u.size());
            debug_assert!(self.unenclosed_empties >= u.size());
            self.unenclosed_empties -= u.size();
          }
        }

        if value == Field::SnakeEnd {
          self.snake_ends.push(pos);
        }
      }
      Field::Empty => {
        assert!(self.empty_allowed(pos));
        self.board[pos] = value;
        self.unenclosed_empties += 1;

        for p in self.board.get_pos_around_4(pos) {
          let u = &self.unions[p];
          u.set_data(u.data() - 1);

          if self.field(p).is_empty() {
            self.unions.merge(pos, p);
          }
        }

        let u = &self.unions[pos];
        if u.data() == 0 {
          self.empty_policy.notify(u.size());
          debug_assert!(self.unenclosed_empties >= u.size());
          self.unenclosed_empties -= u.size();
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
    //if snakes_around == 0 {
    //  return true;
    //}

    if snakes_around > 2 {
      return false;
    }

    let mut first_seg: Option<BoardUnionId> = None;

    struct Cluster {
      size: usize,
      unknown_neighbours: u32,
    }
    impl Cluster {
      fn new(u: &BoardUnion<u32>) -> Self {
        Self {
          size: u.size(),
          unknown_neighbours: u.data(),
        }
      }
    }
    let mut empty_clusters: HashMap<BoardUnionId, Cluster> = HashMap::new();
    let mut policy = self.empty_policy.clone();

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
        let cluster = cluster.or_insert_with(|| Cluster::new(u));
        if cluster.unknown_neighbours <= 1 {
          if !policy.allowed(cluster.size) {
            //println!("{:?}", pos);
            //println!("{:?}", self);
            return false;
          }
          policy.notify(cluster.size);
        }
        cluster.unknown_neighbours -= 1;
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
    self.empty_policy.allowed(cluster_count)
      || self.empty_policy.could_become_allowed(cluster_count) && will_not_be_closed
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

impl Eq for State {}

impl PartialEq for State {
  fn eq(&self, other: &Self) -> bool {
    self.board == other.board && self.empty_policy == other.empty_policy
  }
}

impl Hash for State {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.board.hash(state);
    self.empty_policy.hash(state);
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

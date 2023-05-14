use core::fmt;
use std::cell::Cell;
use std::collections::VecDeque;
use std::ops::{self, Add, Index, IndexMut, Neg, Sub};

use rand::distributions::Standard;
use rand::prelude::Distribution;
use rand::Rng;
use serde::{Deserialize, Serialize};

pub static NORTH: BoardVec = BoardVec::new(0, -1);
pub static NORTH_EAST: BoardVec = BoardVec::new(1, -1);
pub static EAST: BoardVec = BoardVec::new(1, 0);
pub static SOUTH_EAST: BoardVec = BoardVec::new(1, 1);
pub static SOUTH: BoardVec = BoardVec::new(0, 1);
pub static SOUTH_WEST: BoardVec = BoardVec::new(-1, 1);
pub static WEST: BoardVec = BoardVec::new(-1, 0);
pub static NORTH_WEST: BoardVec = BoardVec::new(-1, -1);
pub static CENTER: BoardVec = BoardVec::new(0, 0);

pub static DIRECTIONS_4: [BoardVec; 4] = [NORTH, WEST, EAST, SOUTH];
pub static DIRECTIONS_8: [BoardVec; 8] =
  [NORTH_WEST, NORTH, NORTH_EAST, WEST, EAST, SOUTH_WEST, SOUTH, SOUTH_EAST];
pub static CENTER_AND_DIRECTIONS: [BoardVec; 9] = [
  NORTH_WEST, NORTH, NORTH_EAST, WEST, CENTER, EAST, SOUTH_WEST, SOUTH, SOUTH_EAST,
];

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct BoardVec {
  pub x: i32,
  pub y: i32,
}

impl Serialize for BoardVec {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    serializer.collect_seq([self.x, self.y])
  }
}


impl BoardVec {
  pub const fn new(x: i32, y: i32) -> BoardVec {
    BoardVec { x, y }
  }

  pub fn with_neighbours(self) -> impl Iterator<Item = BoardVec> {
    CENTER_AND_DIRECTIONS.iter().map(move |&dir| dir + self)
  }

  pub fn neighbours_4(self) -> impl Iterator<Item = BoardVec> {
    DIRECTIONS_4.iter().map(move |&dir| dir + self)
  }

  pub fn neighbours_8(self) -> impl Iterator<Item = BoardVec> {
    DIRECTIONS_8.iter().map(move |&dir| dir + self)
  }

  pub fn rand(self, rng: &mut impl Rng) -> Self {
    Self::new(rng.gen_range(0..self.x), rng.gen_range(0..self.y))
  }

  pub fn dist(self, rhs: Self) -> u32 {
    (self.x - rhs.x).unsigned_abs() + (self.y - rhs.y).unsigned_abs()
  }
}

impl fmt::Debug for BoardVec {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "({}, {})", self.x, self.y)
  }
}

impl Add<BoardVec> for BoardVec {
  type Output = BoardVec;

  fn add(self, rhs: BoardVec) -> Self::Output {
    BoardVec::new(self.x + rhs.x, self.y + rhs.y)
  }
}

impl Sub<BoardVec> for BoardVec {
  type Output = BoardVec;

  fn sub(self, rhs: BoardVec) -> Self::Output {
    BoardVec::new(self.x - rhs.x, self.y - rhs.y)
  }
}

impl Neg for BoardVec {
  type Output = BoardVec;

  fn neg(self) -> Self::Output {
    BoardVec::new(-self.x, -self.y)
  }
}

impl Distribution<BoardVec> for Standard {
  fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> BoardVec {
    BoardVec::new(rng.gen(), rng.gen())
  }
}

fn pos_to_index(pos: BoardVec, width: usize, height: usize) -> Option<usize> {
  match (usize::try_from(pos.x), usize::try_from(pos.y)) {
    (Ok(x), Ok(y)) if x < width && y < height => Some(x + y * width),
    _ => None,
  }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Board<T> {
  pub width: u32,
  pub height: u32,
  fields: Vec<T>,
}

impl<T> Board<T> {
  pub fn new(width: u32, height: u32, default: T) -> Self
  where
    T: Clone,
  {
    Self {
      width,
      height,
      fields: vec![default; (width * height) as usize],
    }
  }

  pub fn pos_to_index(&self, pos: BoardVec) -> Option<usize> {
    pos_to_index(pos, self.width as usize, self.height as usize)
  }

  pub fn get(&self, pos: BoardVec) -> Option<&T> {
    self.pos_to_index(pos).and_then(|i| self.fields.get(i))
  }

  pub fn get_mut(&mut self, pos: BoardVec) -> Option<&mut T> {
    self.pos_to_index(pos).and_then(|i| self.fields.get_mut(i))
  }

  pub fn get_around_4(&self, pos: BoardVec) -> impl Iterator<Item = &T> {
    pos.neighbours_4().flat_map(|pos| self.get(pos))
  }

  pub fn get_pos_around_4(&self, pos: BoardVec) -> impl Iterator<Item = BoardVec> + '_ {
    pos.neighbours_4().flat_map(|pos| self.get(pos).and(Some(pos)))
  }

  pub fn get_around_8(&self, pos: BoardVec) -> impl Iterator<Item = &T> {
    pos.neighbours_8().flat_map(|pos| self.get(pos))
  }

  pub fn get_pos_around_8(&self, pos: BoardVec) -> impl Iterator<Item = BoardVec> + '_ {
    pos.neighbours_8().flat_map(|pos| self.get(pos).and(Some(pos)))
  }

  pub fn positions(&self) -> BoardPositionIterator {
    BoardPositionIterator::new(BoardVec::new(0, 0), self.width, self.height)
  }
  pub fn enumerate(&self) -> impl Iterator<Item = (BoardVec, &T)> {
    self.positions().zip(self.fields.iter())
  }

  pub fn enumerate_mut(&mut self) -> impl Iterator<Item = (BoardVec, &mut T)> {
    self.positions().zip(self.fields.iter_mut())
  }

  pub fn iter(&self) -> impl Iterator<Item = &T> {
    self.fields.iter()
  }
}

impl<T> Index<BoardVec> for Board<T> {
  type Output = T;

  fn index(&self, index: BoardVec) -> &Self::Output {
    self.get(index).unwrap_or_else(|| {
      panic!(
        "Cannot access position {:?} on board with size {}x{}",
        index, self.width, self.height
      )
    })
  }
}

impl<T> IndexMut<BoardVec> for Board<T> {
  fn index_mut(&mut self, index: BoardVec) -> &mut T {
    let (width, height) = (self.width, self.height);
    self.get_mut(index).unwrap_or_else(|| {
      panic!(
        "Cannot mut-access position {:?} on board with size {}x{}",
        index, width, height
      )
    })
  }
}

pub struct BoardPositionIterator {
  next_pos: BoardVec,
  x_start: i32,
  x_end: i32,
  y_end: i32,
}

impl BoardPositionIterator {
  pub fn new(pos: BoardVec, width: u32, height: u32) -> Self {
    let y_end = pos.y + height as i32;
    Self {
      next_pos: if width == 0 { BoardVec::new(0, y_end) } else { pos },
      x_start: pos.x,
      x_end: pos.x + width as i32,
      y_end,
    }
  }
}

impl Iterator for BoardPositionIterator {
  type Item = BoardVec;

  fn next(&mut self) -> Option<Self::Item> {
    let pos = &mut self.next_pos;
    if pos.y >= self.y_end {
      None
    } else {
      let result = *pos;
      pos.x += 1;
      if pos.x >= self.x_end {
        pos.x = self.x_start;
        pos.y += 1;
      }
      Some(result)
    }
  }
}

pub struct BoardExplorer {
  queue: VecDeque<BoardVec>,
  visited: Board<bool>,
}

impl BoardExplorer {
  pub fn enqueue(&mut self, pos: BoardVec) -> bool {
    if let Some(field) = self.visited.get_mut(pos) {
      if !*field {
        *field = true;
        self.queue.push_back(pos);
        return true;
      }
    }
    false
  }

  pub fn enqueue_all(&mut self, all: impl IntoIterator<Item = BoardVec>) {
    for pos in all {
      self.enqueue(pos);
    }
  }

  pub fn pop(&mut self) -> Option<BoardVec> {
    self.queue.pop_front()
  }
}

impl<T> From<&Board<T>> for BoardExplorer {
  fn from(board: &Board<T>) -> Self {
    Self {
      queue: VecDeque::new(),
      visited: Board::new(board.width, board.height, false),
    }
  }
}

pub type BoardUnionId = usize;

pub trait UnionFindData: Default + Copy {
  fn merge(a: Self, b: Self) -> Self;
}

impl UnionFindData for () {
  fn merge(_: Self, _: Self) -> Self {}
}

impl UnionFindData for u32 {
  fn merge(a: Self, b: Self) -> Self {
    a + b
  }
}

#[derive(Eq, Clone, Debug)]
pub struct BoardUnion<D: UnionFindData> {
  target: Cell<u32>,
  size: usize,
  data: Cell<D>,
}

impl<D: UnionFindData> BoardUnion<D> {
  fn new(id: u32) -> Self {
    Self {
      target: Cell::new(id),
      size: 1,
      data: Cell::default(),
    }
  }
  pub fn id(&self) -> BoardUnionId {
    self.target.get() as usize
  }

  pub fn size(&self) -> usize {
    self.size
  }

  pub fn data(&self) -> D {
    self.data.get()
  }

  pub fn set_data(&self, data: D) {
    self.data.set(data);
  }
}

impl<D: UnionFindData> PartialEq for BoardUnion<D> {
  fn eq(&self, other: &Self) -> bool {
    self.target == other.target
  }
}

#[derive(Clone)]
pub struct BoardUnionFind<D: UnionFindData = ()> {
  width: u32,
  height: u32,
  fields: Vec<BoardUnion<D>>,
}

impl<D: UnionFindData> BoardUnionFind<D> {
  pub fn new(width: u32, height: u32) -> Self {
    let fields = (0..width * height).map(BoardUnion::new).collect();

    Self { width, height, fields }
  }

  pub fn merge(&mut self, a: BoardVec, b: BoardVec) -> (bool, &BoardUnion<D>) {
    let a = &self[a];
    let b = &self[b];

    if a == b {
      let id = a.id();
      (false, &self[id])
    } else {
      let (sup, sub) = if a.size() >= b.size() { (a, b) } else { (b, a) };

      let data = D::merge(sup.data.get(), sub.data.get());
      let size = sup.size() + sub.size();
      sub.target.set(sup.id() as u32);

      let sup = sup.id();
      let sup = &mut self.fields[sup];
      sup.size = size;
      sup.data.set(data);
      (true, sup)
    }
  }
}

impl<D: UnionFindData> ops::Index<BoardUnionId> for BoardUnionFind<D> {
  type Output = BoardUnion<D>;

  fn index(&self, id: BoardUnionId) -> &Self::Output {
    let union = &self.fields[id];
    let target = union.id();

    if target == id {
      union
    } else {
      let target = &self[target];
      target.target.set(target.target.get());
      target
    }
  }
}

impl<D: UnionFindData> ops::Index<BoardVec> for BoardUnionFind<D> {
  type Output = BoardUnion<D>;

  fn index(&self, pos: BoardVec) -> &Self::Output {
    &self[pos_to_index(pos, self.width as usize, self.height as usize).unwrap() as BoardUnionId]
  }
}

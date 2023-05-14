use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::board::BoardVec;
use crate::{EmptyPolicy, State};

#[derive(Debug, Serialize, Deserialize)]
pub enum SerializableEmptyPolicy {
  None,
  Fix { fix_size: usize },
  Ascending { top: usize },
}

impl SerializableEmptyPolicy {
  pub fn new(p: &EmptyPolicy) -> Self {
    match &p {
      EmptyPolicy::None => Self::None,
      EmptyPolicy::Fix(fix_size) => Self::Fix { fix_size: *fix_size },
      EmptyPolicy::Ascending(nums, _) => Self::Ascending { top: nums.len() },
    }
  }
}

#[derive(Debug, Serialize)]
pub struct LevelData {
  width: usize,
  height: usize,
  max_assumption_depth: usize,
  fields: HashMap<String, char>,
  level: Vec<String>,
  initial_open: Vec<BoardVec>,
  moves: Vec<BoardVec>,
  author: String,
  empty_policy: SerializableEmptyPolicy,
}

impl LevelData {
  pub fn new(
    solution: &State,
    mut initial_open: Vec<BoardVec>,
    moves: Vec<BoardVec>,
    max_assumption_depth: usize,
  ) -> Self {
    let mut fields = HashMap::new();
    fields.insert("snake-head".to_string(), 'X');
    fields.insert("snake-body".to_string(), '+');
    fields.insert("empty".to_string(), '.');

    let mut level = Vec::new();
    for y in 0..solution.height() {
      let mut line = String::new();
      for x in 0..solution.width() {
        line.push(match solution.field(BoardVec::new(x as i32, y as i32)) {
          crate::Field::Unknown => panic!("solution should not contain unknown"),
          crate::Field::Snake => '+',
          crate::Field::SnakeEnd => 'X',
          crate::Field::Empty => '.',
        });
      }
      level.push(line);
    }

    initial_open.extend(solution.snake_ends.iter());

    Self {
      height: solution.height() as usize,
      width: solution.width() as usize,
      max_assumption_depth,
      fields,
      level,
      initial_open,
      moves,
      author: "Tobias K.".to_string(),
      empty_policy: SerializableEmptyPolicy::new(&solution.empty_policy),
    }
  }
}

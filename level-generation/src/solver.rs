use std::collections::hash_map::DefaultHasher;
use std::collections::{BinaryHeap, HashSet};
use std::hash::{Hash, Hasher};
use std::mem;

#[allow(unused_imports)]
use rand::{thread_rng, Rng};

use crate::board::BoardVec;
use crate::list::List;
use crate::{Field, SnakeConnectedness, State, Throwaway};

#[derive(Debug, Clone)]
pub enum FillResult {
  Contradiction,
  Solved(State),
  Ok(State, usize),
}

pub fn fill_obvious(mut state: State, moves: &mut impl Extend<BoardVec>) -> FillResult {
  let mut changes = 0;
  loop {
    let mut changed = false;
    for pos in state.board.positions() {
      let field = state.field(pos);
      if field.is_snake()
        && state.is_dangling_snake(pos)
        && state.unknown_around(pos) < field.max_snake_neighbours() - state.snakes_around(pos)
      {
        return FillResult::Contradiction;
      }

      if field != Field::Unknown {
        continue;
      }

      let snake_allowed = state.snake_allowed(pos);
      let empty_allowed = state.empty_allowed(pos);

      if !snake_allowed && !empty_allowed {
        return FillResult::Contradiction;
      } else if !snake_allowed {
        state.set(pos, Field::Empty);
        moves.extend([pos]);
        changes += 1;
        changed = true;
      } else if !empty_allowed {
        state.set(pos, Field::Snake);
        moves.extend([pos]);
        changes += 1;
        changed = true;
      }
    }
    if !changed {
      break;
    }
  }

  let connected = state.is_snake_connected();

  if connected == SnakeConnectedness::Distributed
    || !state
      .empty_policy
      .is_still_possible(state.unenclosed_empties + state.unknowns as usize)
  {
    FillResult::Contradiction
  } else if state.unknowns == 0 {
    if connected == SnakeConnectedness::Connected {
      FillResult::Solved(state)
    } else {
      FillResult::Contradiction
    }
  } else {
    FillResult::Ok(state, changes)
  }
}

pub fn solve(state: State, results: &mut Vec<State>, max_results: usize) {
  if results.len() >= max_results {
    return;
  }

  //println!("{:?}", state);
  let state = match fill_obvious(state, &mut Throwaway) {
    FillResult::Contradiction => return,
    FillResult::Solved(state) => {
      results.push(state);
      return;
    }
    FillResult::Ok(state, _) => state,
  };

  if let Some(pos) = state.board.positions().find(|&p| state.field(p) == Field::Unknown) {
    {
      let mut s = state.clone();
      s.set(pos, Field::Snake);
      solve(s, results, max_results);
    }

    {
      let mut s = state;
      s.set(pos, Field::Empty);
      solve(s, results, max_results)
    }
  }
}

#[derive(Debug, Clone)]
struct Item {
  initial_open_count: usize,
  state: State,
  moves: List<BoardVec>,
  initial_open: List<BoardVec>,
  finished: bool,
}

impl Item {
  fn new(state: State) -> Self {
    assert!(state.unknowns() > 0);
    Self {
      initial_open_count: 0,
      state,
      moves: List::nil(),
      initial_open: List::nil(),
      finished: false,
    }
  }

  fn fingerprint(&self) -> u64 {
    let hasher = &mut DefaultHasher::new();
    self.state.hash(hasher);
    self.initial_open_count.hash(hasher);
    for m in self.moves.iter() {
      let move_hasher = &mut DefaultHasher::new();
      m.hash(move_hasher);
      move_hasher.finish().hash(hasher);
    }
    hasher.finish()
  }

  fn with_move(self, pos: BoardVec, field: Field) -> Self {
    let Item {
      initial_open_count,
      mut state,
      mut moves,
      initial_open,
      finished,
    } = self;

    state.set(pos, field);
    moves.push(pos);

    Self {
      initial_open_count,
      state,
      moves,
      initial_open,
      finished,
    }
  }

  fn with_opened(self, pos: BoardVec, solution: &State) -> Self {
    let Item {
      mut initial_open_count,
      mut state,
      moves,
      mut initial_open,
      finished,
    } = self;

    state.set(pos, solution.field(pos));
    initial_open_count += 1;
    initial_open.push(pos);

    Self {
      initial_open_count,
      state,
      moves,
      initial_open,
      finished,
    }
  }

  fn with_filled(self, solution: &State) -> Self {
    let Item {
      initial_open_count,
      state,
      mut moves,
      initial_open,
      ..
    } = self;

    let (state, finished) = match fill_obvious(state.clone(), &mut moves) {
      FillResult::Contradiction => {
        println!("Solution:\n{:?}", solution);
        println!("State:\n{:?}", state);
        fill_obvious(state, &mut Throwaway);
        panic!("No item should ever be in contradiction!")
      }
      FillResult::Solved(state) => (state, true),
      FillResult::Ok(state, _) => (state, false),
    };

    Self {
      initial_open_count,
      state,
      moves,
      initial_open,
      finished,
    }
  }
}

impl Eq for Item {}

impl PartialEq for Item {
  fn eq(&self, other: &Self) -> bool {
    self.state.unknowns == other.state.unknowns
  }
}

impl Ord for Item {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    self.state.unknowns.cmp(&other.state.unknowns).reverse()
  }
}

impl PartialOrd for Item {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    Some(self.cmp(other))
  }
}

pub fn find_solution_path(
  begin: State,
  solution: &State,
  max_assume_depth: usize,
) -> (Vec<BoardVec>, Vec<BoardVec>) {
  let mut items = BinaryHeap::new();
  items.push(Item::new(begin));
  let item = find_solution_path2(solution, items, max_assume_depth);
  (
    item.initial_open.iter().cloned().collect(),
    item.moves.iter().cloned().collect(),
  )
}

fn find_solution_path2(solution: &State, mut items: BinaryHeap<Item>, max_depth: usize) -> Item {
  let mut fingerprints = HashSet::new();
  loop {
    assert!(!items.is_empty());
    println!("Items in queue {}", items.len());
    for item in mem::take(&mut items).drain_sorted().take(100) {
      println!(
        "Item(opened: {}, unknowns: {})",
        item.initial_open_count, item.state.unknowns
      );
      if item.finished {
        return item;
      }

      //let mut pushed_one = false;
      for pos in item.state.board.positions() {
        //print!("{:?}", pos);
        if item.state.field(pos) == Field::Unknown && solution.field(pos) == Field::Empty {
          //&& (!pushed_one || thread_rng().gen_range(0..=(items.len() / 200)) == 0) {
          let new_item = item.clone().with_opened(pos, solution);
          if let Ok(new_item) = further_item_multi(new_item, max_depth, solution) {
            if fingerprints.insert(new_item.fingerprint()) {
              items.push(new_item);
              //pushed_one = true;
            }
          }
        }
      }
    }
  }
}

fn further_item_multi(mut item: Item, max_depth: usize, solution: &State) -> Result<Item, Item> {
  let mut furthered = false;
  loop {
    item = match further_item(item, max_depth, solution) {
      Ok(item) => item,
      Err(item) if furthered => return Ok(item),
      Err(item) => return Err(item),
    };

    furthered = true;
  }
}

fn further_item(item: Item, max_depth: usize, solution: &State) -> Result<Item, Item> {
  let moves_before_fill = item.moves.clone();
  let item = item.with_filled(solution);
  if max_depth > 0 {
    let state = &item.state;
    for pos in state.board.positions() {
      if state.field(pos) == Field::Unknown {
        let res_snake = {
          let mut s = state.clone();
          s.set(pos, Field::Snake);
          find_contradiction(s, max_depth, pos)
        };

        match res_snake {
          FindContradictionResult::Contradiction => {
            return Ok(item.with_move(pos, Field::Empty).with_filled(solution));
          }
          FindContradictionResult::Solved => {
            return Ok(item.with_move(pos, Field::Snake).with_filled(solution));
          }
          FindContradictionResult::None => (),
        }

        let res_empty = {
          let mut s = state.clone();
          s.set(pos, Field::Empty);
          find_contradiction(s, max_depth, pos)
        };

        match res_empty {
          FindContradictionResult::Contradiction => {
            return Ok(item.with_move(pos, Field::Snake).with_filled(solution));
          }
          FindContradictionResult::Solved => {
            return Ok(item.with_move(pos, Field::Empty).with_filled(solution));
          }
          FindContradictionResult::None => (),
        }
      }
    }
  }

  if moves_before_fill.len() == item.moves.len() {
    Err(item)
  } else {
    Ok(item)
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FindContradictionResult {
  Contradiction,
  Solved,
  None,
}

fn find_contradiction(state: State, rest_depth: usize, last_pos: BoardVec) -> FindContradictionResult {
  //println!("{:?}", state);
  let state = match fill_obvious(state, &mut Throwaway) {
    FillResult::Contradiction => return FindContradictionResult::Contradiction,
    FillResult::Solved(_) => return FindContradictionResult::Solved,
    FillResult::Ok(state, _) => state,
  };

  if rest_depth == 0 {
    return FindContradictionResult::None;
  }

  for pos in state.pos_around(last_pos) {
    if state.field(pos) == Field::Unknown {
      let res_snake = {
        let mut s = state.clone();
        s.set(pos, Field::Snake);
        find_contradiction(s, rest_depth - 1, pos)
      };

      if res_snake == FindContradictionResult::Solved {
        return FindContradictionResult::Solved;
      } else if res_snake == FindContradictionResult::None {
        return FindContradictionResult::None;
      }

      let res_empty = {
        let mut s = state.clone();
        s.set(pos, Field::Empty);
        find_contradiction(s, rest_depth - 1, pos)
      };

      match (res_snake, res_empty) {
        (FindContradictionResult::Contradiction, FindContradictionResult::Contradiction) => {
          return FindContradictionResult::Contradiction
        }
        (_, FindContradictionResult::Solved) => return FindContradictionResult::Solved,
        _ => (),
      }
    }
  }

  FindContradictionResult::None
}

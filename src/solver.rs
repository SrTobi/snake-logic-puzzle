use crate::{Field, SnakeConnectedness, State};

#[derive(Debug, Clone)]
pub enum FillResult {
  Contradiction,
  Solved,
  Ok(State),
}

pub fn fill_obvious(mut state: State, results: &mut Vec<State>) -> FillResult {
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

  let connected = state.is_snake_connected();

  if connected == SnakeConnectedness::Distributed || !state.empty_policy.is_still_possible(state.unenclosed_empties) {
    FillResult::Contradiction
  } else if state.unknowns == 0 {
    if connected == SnakeConnectedness::Connected {
      results.push(state);
      FillResult::Solved
    } else {
      FillResult::Contradiction
    }
  } else {
    FillResult::Ok(state)
  }
}

pub fn solve(state: State, results: &mut Vec<State>, max_results: usize) {
  if results.len() >= max_results {
    return;
  }

  //println!("{:?}", state);
  let state = match fill_obvious(state, results) {
    FillResult::Contradiction | FillResult::Solved => return,
    FillResult::Ok(state) => state,
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

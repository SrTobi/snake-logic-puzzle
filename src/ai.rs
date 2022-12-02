/* 
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum FieldKnowledge {
  Unknown,
  Snake,
  SnakeEnd,
  Empty(u32),
}

*/




/*use crate::board::{Board, BoardVec};
use crate::{Field, Game};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum FieldKnowledge {
  Unknown,
  Mine,
  Explored(u32, u32),
}

use FieldKnowledge::*;

#[derive(Clone, PartialEq, Eq, Hash)]
struct State {
  board: Board<FieldKnowledge>,
  mines_left: u32,
}

impl State {
  fn mark_mine(&mut self, pos: BoardVec, mut actions: Option<&mut Vec<BoardVec>>) {
    match self.board[pos] {
      Unknown => {
        self.board[pos] = Mine;
        for neighbour_pos in pos.neighbours() {
          if let Some(Explored(_, mine_neighbours_left)) = self.board.get_mut(neighbour_pos) {
            debug_assert!(*mine_neighbours_left > 0);
            *mine_neighbours_left -= 1;
            if let Some(actions) = &mut actions {
              self.check_for_action(pos, actions);
            }
          }
        }
      }
      Mine => (),
      Explored(_, _) => panic!("should not mark an explored field as mine."),
    }
  }

  fn mark_explored(&mut self, pos: BoardVec, game: &Game) {
    match self.board[pos] {
      Unknown => {
        let field = game.view(pos).expect("Explored field should be visible in game");
        if let Field::Empty(n) = field {
          self.board[pos] = Explored(n, pos.neighbours().filter(|p| self.board[*p] == Mine).count() as u32);
        } else {
          panic!("Cannot explore fields with mines on.")
        }
      }
      Mine => panic!("Cannot mark a field with a mine as explored"),
      Explored(_, _) => panic!("Already marked as explored"),
    }
  }

  fn search_action(&self, actions: &mut Vec<BoardVec>) {
    for pos in self.board.positions() {
      self.check_for_action(pos, actions);
    }
  }

  fn check_for_action(&self, pos: BoardVec, actions: &mut Vec<BoardVec>) {
    if let Explored(n, 0) = self.board[pos] {
      if n > 0 {
        for neighbour_pos in pos.neighbours() {
          if let Some(Unknown) = self.board.get(neighbour_pos) {
            actions.push(neighbour_pos);
          }
        }
      }
    }
  }
}

impl From<&Game> for State {
  fn from(game: &Game) -> Self {
    let mut result = Self {
      board: Board::new(game.width(), game.height(), Unknown),
      mines_left: game.setup().mines,
    };

    result
  }
}
*/
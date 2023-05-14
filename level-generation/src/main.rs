use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};

use snake::board::BoardVec;
use snake::serialize::LevelData;
use snake::{find_solution_path, solve, Field, State};

#[allow(unused)]
fn main() {
  let width = 8;
  let height = 8;

  let mut a = 0;
  loop {
    let game = State::new_rand(width, height, snake::EmptyPolicy::new_ascending(width, height));
    let mut results = Vec::new();
    solve(game.clone(), &mut results, 2);

    if !results.is_empty() {
      for ele in results.iter() {
        println!("{:?}", ele);
      }

      let solution = results.first().unwrap();
      show_solution(&game, solution, 1);

    } else {
      println!("faild ({a})...");
      a += 1;
    }
  }
}

/*#[allow(unused)]
fn main2() {
  let game = State::new_rand(10, 10, snake::EmptyPolicy::Fix(5)); //new(11, 11, BoardVec::new(7, 8), BoardVec::new(8, 10));
  println!("{:?}", game);

  let mut s = Vec::new();
  let r = solve(game, usize::MAX / 2, &mut s);
  println!("{:?}", r);

  if let Ok(SolveResult::Contradiction) = r {
    s.sort_by_key(|s| u32::MAX - s.1.unknowns());

    for (p, s) in s {
      println!("{:?}", p);
      println!("{:?}", s);
    }
  }
}*/

#[allow(unused)]
fn main3() {
  let mut game = State::new(
    11,
    11,
    BoardVec::new(7, 8),
    BoardVec::new(8, 10),
    snake::EmptyPolicy::Fix(5),
  );

  let board = "
  x-----------x
  |···+++··+++|
  |·+++·+·++·+|
  |·+···+·+··+|
  |++·+++·+·++|
  |+·++··++·+·|
  |+·+··++·++·|
  |+·++·+··+··|
  |+··+++··++·|
  |+++···+X·++|
  |··++··+···+|
  |···++++·X++|
  x-----------x
  ";

  let mut it = board.chars().filter(|&c| "+·".contains(c));

  for y in 0..11 {
    for x in 0..11 {
      let v = BoardVec::new(x, y);

      if game.field(v) == Field::Unknown {
        println!("{:?}", game);
        game.set(
          v,
          match it.next().unwrap() {
            '·' => Field::Empty,
            '+' => Field::Snake,
            _ => unreachable!(),
          },
        );
      }
    }
  }
}

#[allow(unused)]
fn main5() {
  let game = State::new(
    10,
    10,
    BoardVec::new(2, 0),
    BoardVec::new(0, 2),
    snake::EmptyPolicy::new_ascending(10, 10),
  ); //new(11, 11, BoardVec::new(7, 8), BoardVec::new(8, 10));
  println!("{:?}", game);

  let mut results = Vec::new();
  //let mut s = Vec::new();
  solve(game.clone(), &mut results, 10);
  let solution = results.last().unwrap();
  println!("{:?}", solution);

  //show_solution(&game, solution);
  /*if let Ok(SolveResult::Contradiction) = r {
    s.sort_by_key(|s| u32::MAX - s.1.unknowns());

    for (p, s) in s {
      println!("{:?}", p);
      println!("{:?}", s);
    }
  }*/
}

fn show_solution(initial: &State, solution: &State, max_assume_depth: usize) {
  let (initial_open, moves) = find_solution_path(initial.clone(), solution, max_assume_depth);

  let mut state = initial.clone();

  for &pos in initial_open.iter() {
    state.set(pos, solution.field(pos));
  }

  println!("{:?}", state);

  let level = LevelData::new(solution, initial_open, moves, max_assume_depth);
  let serialized = serde_json::to_string_pretty(&level).unwrap();
  println!("{serialized}");

  let hasher = &mut DefaultHasher::new();
  initial.hash(hasher);
  let filename = format!(
    "./level_out/level_{}x{}_{}_{}.json",
    initial.width(),
    initial.height(),
    max_assume_depth,
    hasher.finish()
  );
  let _ = fs::create_dir("./level_out");
  fs::write(filename, serialized).unwrap();
}

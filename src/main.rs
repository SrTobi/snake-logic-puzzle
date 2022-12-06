use snake::board::{Board, BoardVec};
use snake::{solve, Field, SolveResult, State};

fn main() {
  loop {
    let game = State::new_rand(10, 10);
    let mut fails = Vec::new();
    let r = solve(game, usize::MAX / 2, &mut fails);

    if let Err(res) = r {
      println!("{:?}", res);
    }
  }
}

fn main2() {
  let game = State::new_rand(10, 10); //new(11, 11, BoardVec::new(7, 8), BoardVec::new(8, 10));
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
}

fn main3() {
  let mut game = State::new(11, 11, BoardVec::new(7, 8), BoardVec::new(8, 10));

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

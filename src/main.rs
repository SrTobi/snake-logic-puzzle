use snake::board::{Board, BoardVec};
use snake::{solve, Field, SolveResult, State};

fn main4() {
  let width = 10;
  let height = 10;

  let mut a = 0;
  loop {
    let game = State::new_rand(width, height, snake::EmptyPolicy::new_ascending(width, height));
    let mut fails = Vec::new();
    let r = solve(game, usize::MAX / 2, &mut fails);

    if let Err(res) = r {
      println!("{:?}", res);
      return;
    } else {
      println!("faild ({a})...");
      a += 1;
    }
  }
}

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
}

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

struct Throwaway;

impl<T> Extend<T> for Throwaway {
  fn extend<I: IntoIterator<Item = T>>(&mut self, _: I) {}
}

fn main() {
  let game = State::new(
    10,
    10,
    BoardVec::new(2, 0),
    BoardVec::new(0, 2),
    snake::EmptyPolicy::new_ascending(10, 10),
  ); //new(11, 11, BoardVec::new(7, 8), BoardVec::new(8, 10));
  println!("{:?}", game);

  //let mut s = Vec::new();
  let r = solve(game, usize::MAX / 2, &mut Throwaway);
  println!("{:?}", r);

  /*if let Ok(SolveResult::Contradiction) = r {
    s.sort_by_key(|s| u32::MAX - s.1.unknowns());

    for (p, s) in s {
      println!("{:?}", p);
      println!("{:?}", s);
    }
  }*/
}

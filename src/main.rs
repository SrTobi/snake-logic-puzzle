use snake::Game;

fn main() {
  let mut game = Game::new_rand(12, 12); //, BoardVec::new(1, 2), BoardVec::new(3, 2));
  println!("{:?}", game);

  let mut i = 0;
  loop {
    i += 1;
    let (complete, complete_old) = game.evolve();
    //println!("{}:\n{:?}", i, game);

    if complete_old {
      println!("{}:\n{:?}", i, game);
    }

    if complete && i > 10 {
      break;
    }
  };
  println!("{}:\n{:?}", i, game);
}

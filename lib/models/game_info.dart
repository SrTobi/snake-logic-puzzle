import 'package:logic_snake_puzzle/utils/board.dart';

enum Field {
  Empty,
  Snake,
  SnakeEnd,
}

class GameInfo {
  final Board<Field> solution;
  final Board<bool> opened;

  const GameInfo({required this.solution, required this.opened});
}

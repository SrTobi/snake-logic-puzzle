class BoardVec {
  final int x;
  final int y;

  static const BoardVec south = BoardVec(0, 1);
  static const BoardVec north = BoardVec(0, -1);
  static const BoardVec west = BoardVec(-1, 0);
  static const BoardVec east = BoardVec(1, 0);
  static const List<BoardVec> directions = [north, east, south, west];

  const BoardVec(this.x, this.y);

  BoardVec operator +(BoardVec v) => BoardVec(x + v.x, y + v.y);
  BoardVec operator -(BoardVec v) => BoardVec(x - v.x, y - v.y);
}

class Board<T> {
  final int width;
  final int height;
  final List<T> _fields;

  Board(this.width, this.height, T Function() init) : _fields = List.generate(width * height, (index) => init());

  T operator [](BoardVec pos) => get(pos)!;

  int index(BoardVec pos) => pos.y * width + pos.x;

  T? get(BoardVec pos) {
    int i = index(pos);

    if (i < 0 || i >= _fields.length) {
      return null;
    } else {
      return _fields[i];
    }
  }
}

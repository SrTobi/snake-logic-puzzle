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

  @override
  bool operator ==(Object other) => other is BoardVec && x == other.x && y == other.y;

  @override
  int get hashCode => Object.hash(x, y);
}

class Board<T> {
  final int width;
  final int height;
  final List<T> _fields;

  Board(this.width, this.height, T Function(BoardVec) init)
      : _fields = List.generate(width * height, (index) => init(calculate_pos(index, width)));

  T operator [](BoardVec pos) => get(pos)!;
  void operator []=(BoardVec pos, T value) => _fields[index(pos)] = value;

  Iterable<BoardVec> positionsAround(BoardVec pos) {
    return BoardVec.directions.map((p) => p + pos).where(contains);
  }

  Iterable<T> itemsAround(BoardVec pos) {
    return positionsAround(pos).map((p) => this[p]);
  }

  bool contains(BoardVec pos) {
    return 0 <= pos.x && pos.x < width && 0 <= pos.y && pos.y < height;
  }

  int index(BoardVec pos) => pos.y * width + pos.x;

  T? get(BoardVec pos) {
    if (contains(pos)) {
      int i = index(pos);
      return _fields[i];
    } else {
      return null;
    }
  }

  static int calculate_index(BoardVec pos, int width) => pos.y * width + pos.x;
  static BoardVec calculate_pos(int index, int width) => BoardVec(index % width, index ~/ width);
}

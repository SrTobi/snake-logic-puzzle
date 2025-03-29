import 'package:snake_logic_puzzle/utils/board.dart';

enum Field {
  unknown(-1),
  empty(-1),
  snake(2),
  snakeHead(1);

  final int targetNeighbourSnakes;

  const Field(this.targetNeighbourSnakes);
}

extension FieldExt on Field {
  bool get isSnake => this == Field.snake || this == Field.snakeHead;
}

abstract class EmptyPolicy {
  const EmptyPolicy();
}

class FixEmptyPolicy extends EmptyPolicy {
  final int fields;

  const FixEmptyPolicy({required this.fields});
}

class AscendingEmptyPolicy extends EmptyPolicy {
  final int top;

  const AscendingEmptyPolicy({required this.top});
}

class GameInfo {
  final Board<Field> solution;
  final List<BoardVec> initialOpen;
  final List<BoardVec> solveMoves;
  final EmptyPolicy emptyPolicy;
  final int maxAssumptionDepth;

  int get width => solution.width;
  int get height => solution.height;

  const GameInfo({
    required this.maxAssumptionDepth,
    required this.solution,
    required this.initialOpen,
    required this.solveMoves,
    required this.emptyPolicy,
  });

  static GameInfo loadFromJson(Map<String, dynamic> json) {
    int width = json["width"];
    int height = json["height"];
    int maxAssumptionDepth = json["max_assumption_depth"];

    int field(String name) => (json["fields"][name] as String).codeUnitAt(0);

    int snakeBodyField = field("snake-body");
    int snakeHeadField = field("snake-head");
    int emptyField = field("empty");

    final solution = Board(width, height, (_) => Field.empty);

    for (int y = 0; y < height; ++y) {
      final line = (json["level"][y] as String).codeUnits;

      if (line.length != width) {
        throw Exception("Wrong line length. Got ${line.length}, expected $width");
      }

      for (int x = 0; x < width; ++x) {
        final field = line[x];
        if (field == emptyField) {
          solution[BoardVec(x, y)] = Field.empty;
        } else if (field == snakeHeadField) {
          solution[BoardVec(x, y)] = Field.snakeHead;
        } else if (field == snakeBodyField) {
          solution[BoardVec(x, y)] = Field.snake;
        } else {
          throw Exception("Unknown field character '${String.fromCharCode(field)}' ($field)");
        }
      }
    }

    BoardVec toVec(dynamic v) {
      final list = v as List<dynamic>;
      assert(list.length == 2);
      return BoardVec(list[0] as int, list[1] as int);
    }

    final initialOpen = (json["initial_open"] as List<dynamic>).map(toVec).toList(growable: false);
    final solveMoves = (json["moves"] as List<dynamic>).map(toVec).toList(growable: false);

    EmptyPolicy? emptyPolicy;
    final emptyPolicyRoot = json["empty_policy"];
    final ascending = emptyPolicyRoot["Ascending"];

    if (ascending != null) {
      emptyPolicy = AscendingEmptyPolicy(top: ascending["top"] as int);
    }

    if (emptyPolicy == null) {
      throw Exception("No empty policy in level data");
    }

    return GameInfo(
      maxAssumptionDepth: maxAssumptionDepth,
      solution: solution,
      initialOpen: initialOpen,
      solveMoves: solveMoves,
      emptyPolicy: emptyPolicy,
    );
  }
}

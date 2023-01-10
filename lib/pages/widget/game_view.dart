import 'dart:collection';

import 'package:flutter/material.dart';
import 'package:flutter_mobx/flutter_mobx.dart';
import 'package:logic_snake_puzzle/models/game_info.dart';
import 'package:logic_snake_puzzle/utils/board.dart';
import 'package:mobx/mobx.dart';

abstract class _EmptyPolicyTracker {
  void notifyNewEnclosedEmpties(int num);
  void notifyRemoveEnclosedEmpties(int num);

  bool isOk(int num);

  Widget info(BuildContext context);

  static _EmptyPolicyTracker from(GameInfo info) {
    final ep = info.emptyPolicy;

    if (ep is FixEmptyPolicy) {
      return _FixedEmptyPolicyTracker(ep.fields);
    } else if (ep is AscendingEmptyPolicy) {
      return _AscendingEmptyPolicyTracker(ep.top);
    }
    throw Exception("Unknown empty policy!");
  }
}

class _AscendingEmptyPolicyTracker extends _EmptyPolicyTracker {
  final int top;
  final ObservableList<int> used;

  _AscendingEmptyPolicyTracker(this.top) : used = ObservableList.of(List.filled(top, 0));

  @override
  bool isOk(int num) => num > 0 && num <= top && used[num - 1] <= 1;

  @override
  void notifyNewEnclosedEmpties(int num) {
    if (num > 0 && num <= top) {
      used[num - 1]++;
    }
  }

  @override
  void notifyRemoveEnclosedEmpties(int num) {
    if (num > 0 && num <= top) {
      assert(used[num - 1] > 0);
      used[num - 1]--;
    }
  }

  @override
  Widget info(BuildContext context) {
    return Column(
      children: [
        const Text("Connected empty fields should have size:"),
        Observer(builder: (context) {
          return Row(
            mainAxisSize: MainAxisSize.min,
            children: List.generate(
              top,
              (i) {
                final style = used[i] >= 1 ? const TextStyle(decoration: TextDecoration.lineThrough) : null;
                return Padding(
                  padding: const EdgeInsets.symmetric(horizontal: 4),
                  child: Text.rich(TextSpan(text: "${i + 1}", style: style)),
                );
              },
            ),
          );
        }),
      ],
    );
  }
}

class _FixedEmptyPolicyTracker extends _EmptyPolicyTracker {
  final int fields;

  _FixedEmptyPolicyTracker(this.fields);

  @override
  bool isOk(int num) => num == fields;

  @override
  void notifyNewEnclosedEmpties(int num) {}

  @override
  void notifyRemoveEnclosedEmpties(int num) {}

  @override
  Widget info(BuildContext context) {
    // TODO: implement info
    throw UnimplementedError();
  }
}

class _FieldInfoVersionHolder {
  int _version;

  _FieldInfoVersionHolder() : _version = 0;

  int next() {
    _version += 1;
    return _version;
  }
}

class FieldInfo {
  final BoardVec pos;
  final _FieldInfoVersionHolder _versionHolder;
  final Observable<Field> _field;
  final Observable<int> _version;
  final Observable<int> _enclosedEmpty;
  final _EmptyPolicyTracker emptyPolicyTracker;
  bool locked = false;

  FieldInfo._(this.pos, this._versionHolder, this.emptyPolicyTracker)
      : _field = Observable(Field.unknown),
        _version = Observable(0),
        _enclosedEmpty = Observable(0);

  int? get enclosedEmpty {
    final num = _enclosedEmpty.value;
    return num == 0 ? null : num;
  }

  bool get enclosedEmptyOk {
    final num = _enclosedEmpty.value;
    return num <= 0 || emptyPolicyTracker.isOk(num);
  }

  int get version => _version.value;
  Field get field => _field.value;

  set field(Field field) {
    _field.value = field;
    _version.value = _versionHolder.next();
  }

  void markEnclosed(int num) {
    assert(num >= 1);
    _enclosedEmpty.value = num;
  }

  void markUnenclosed() {
    _enclosedEmpty.value = 0;
  }
}

class AllConnectedEmptyResult {
  final List<FieldInfo> fields;
  final bool enclosed;

  const AllConnectedEmptyResult._(this.fields, {required this.enclosed});
}

class GameViewModel {
  final GameInfo gameInfo;
  final Board<FieldInfo> fields;

  GameViewModel({required this.gameInfo})
      : fields = Board(gameInfo.width, gameInfo.height, _fieldInfoFactory(gameInfo)) {
    for (final open in gameInfo.initialOpen) {
      fields[open].field = gameInfo.solution[open];
      fields[open].locked = true;
    }
  }

  _EmptyPolicyTracker get _emptyPolicyTracker => fields[const BoardVec(0, 0)].emptyPolicyTracker;

  void set(BoardVec pos, Field newField) {
    final cur = fields[pos];

    if (cur.field != Field.unknown) {
      if (newField == Field.empty) {
        unmarkEmpty(pos);
      } else {
        fields.positionsAround(pos).forEach(unmarkEmpty);
      }
    }

    cur.field = newField;

    if (newField == Field.empty) {
      markEmpty(pos);
    } else {
      fields.positionsAround(pos).forEach(markEmpty);
    }
  }

  bool isConnectionInvalid(FieldInfo a, FieldInfo b) {
    bool inner(FieldInfo a, FieldInfo b) {
      final snakesAround = fields.itemsAround(a.pos).where((item) => item.field.isSnake);
      int smallerVersions = 0;
      for (final snake in snakesAround) {
        if (snake.version < b.version) {
          ++smallerVersions;
          if (smallerVersions >= a.field.targetNeighbourSnakes) {
            return true;
          }
        }
      }
      return false;
    }

    return inner(a, b) || inner(b, a);
  }

  void markEmpty(BoardVec pos) {
    if (fields[pos].enclosedEmpty != null) {
      return;
    }

    final result = allConnectedEmpty(pos);
    if (result.enclosed) {
      final empties = result.fields.length;
      _emptyPolicyTracker.notifyNewEnclosedEmpties(empties);

      for (final field in result.fields) {
        field.markEnclosed(empties);
      }
    }
  }

  void unmarkEmpty(BoardVec pos) {
    final enclosedEmpty = fields[pos].enclosedEmpty;
    if (enclosedEmpty == null) {
      return;
    }

    _emptyPolicyTracker.notifyRemoveEnclosedEmpties(enclosedEmpty);

    final result = allConnectedEmpty(pos);
    for (final field in result.fields) {
      field.markUnenclosed();
    }
  }

  AllConnectedEmptyResult allConnectedEmpty(BoardVec start) {
    final List<FieldInfo> result = [];
    final Queue<FieldInfo> queue = Queue();
    final Set<BoardVec> visited = {};
    bool enclosed = true;

    void add(FieldInfo f) {
      if (visited.add(f.pos)) {
        queue.add(f);
      }
    }

    add(fields[start]);

    while (queue.isNotEmpty) {
      final cur = queue.removeFirst();

      if (cur.field != Field.empty) {
        if (cur.field == Field.unknown) {
          enclosed = false;
        }
        continue;
      }

      result.add(cur);

      fields.itemsAround(cur.pos).forEach(add);
    }

    return AllConnectedEmptyResult._(result, enclosed: enclosed);
  }

  static FieldInfo Function(BoardVec) _fieldInfoFactory(GameInfo info) {
    final versionHolder = _FieldInfoVersionHolder();
    final emptyPolicyTracker = _EmptyPolicyTracker.from(info);
    return (pos) => FieldInfo._(pos, versionHolder, emptyPolicyTracker);
  }
}

class EmptyPolicyDescriptionView extends StatelessWidget {
  final GameViewModel model;
  const EmptyPolicyDescriptionView({super.key, required this.model});

  @override
  Widget build(BuildContext context) {
    return model._emptyPolicyTracker.info(context);
  }
}

class GameView extends StatefulWidget {
  final GameViewModel model;
  final void Function(FieldInfo) onClick;

  const GameView({super.key, required this.model, required this.onClick});

  @override
  State<GameView> createState() => _GameViewState();
}

class _GameViewState extends State<GameView> {
  GameViewModel get model => widget.model;
  GameInfo get info => model.gameInfo;

  @override
  Widget build(BuildContext context) {
    List<Row> grid = List.empty(growable: true);

    for (int y = 0; y < info.height; ++y) {
      final List<Widget> row = List.empty(growable: true);
      for (int x = 0; x < info.width; ++x) {
        row.add(_GameViewCell(model, BoardVec(x, y), widget.onClick));
      }
      grid.add(Row(mainAxisSize: MainAxisSize.min, children: row));
    }

    return Column(mainAxisSize: MainAxisSize.min, children: grid);
  }
}

class _GameViewCell extends StatelessWidget {
  final GameViewModel model;
  final void Function(FieldInfo) onClick;
  final BoardVec pos;

  const _GameViewCell(this.model, this.pos, this.onClick);

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: () => onClick(model.fields[pos]),
      child: SizedBox(
        width: 40,
        height: 40,
        child: Container(
          decoration: BoxDecoration(border: Border.all(color: const Color(0xFF888888))),
          child: Observer(builder: (context) {
            final fieldInfo = model.fields[pos];
            switch (fieldInfo.field) {
              case Field.unknown:
                return const SizedBox.expand();
              case Field.empty:
                final enclosedEmpty = fieldInfo.enclosedEmpty;
                return Stack(
                  fit: StackFit.expand,
                  children: [
                    ColoredBox(color: fieldInfo.locked ? const Color(0xFF999999) : const Color(0xFFAAAAAA)),
                    if (enclosedEmpty != null)
                      Center(
                        child: Text(
                          enclosedEmpty.toString(),
                          style: TextStyle(fontSize: 20, color: fieldInfo.enclosedEmptyOk ? Colors.black : Colors.red),
                        ),
                      ),
                  ],
                );
              case Field.snake:
                final connectedFields = model.fields.itemsAround(pos).where((item) => item.field.isSnake).toList();
                connectedFields.sort((a, b) => a.version.compareTo(b.version));

                final connections = connectedFields.map((field) {
                  final diff = field.pos - pos;

                  return Padding(
                    padding: EdgeInsets.only(
                      left: diff.x < 0 ? 0 : 10,
                      top: diff.y < 0 ? 0 : 10,
                      right: diff.x > 0 ? 0 : 10,
                      bottom: diff.y > 0 ? 0 : 10,
                    ),
                    child: ColoredBox(
                      color: model.isConnectionInvalid(fieldInfo, field) ? Colors.red : Colors.black,
                    ),
                  );
                });

                return Stack(
                  fit: StackFit.expand,
                  children: [
                    ...connections,
                    const Center(
                      child: SizedBox.square(
                        dimension: 24,
                        child: ColoredBox(color: Colors.black),
                      ),
                    ),
                  ],
                );
              case Field.snakeHead:
                return const ColoredBox(color: Colors.black);
            }
          }),
        ),
      ),
    );
  }
}

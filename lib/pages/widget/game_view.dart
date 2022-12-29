import 'package:flutter/cupertino.dart';
import 'package:flutter/material.dart';
import 'package:flutter_mobx/flutter_mobx.dart';
import 'package:logic_snake_puzzle/models/game_info.dart';
import 'package:logic_snake_puzzle/utils/board.dart';
import 'package:mobx/mobx.dart';

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
  bool locked = false;

  FieldInfo._(this.pos, this._versionHolder)
      : _field = Observable(Field.unknown),
        _version = Observable(0);

  int get version => _version.value;
  Field get field => _field.value;

  set field(Field field) {
    _field.value = field;
    _version.value = _versionHolder.next();
  }
}

class GameViewModel {
  final GameInfo gameInfo;
  final Board<FieldInfo> fields;

  GameViewModel({required this.gameInfo}) : fields = Board(gameInfo.width, gameInfo.height, _fieldInfoFactory()) {
    for (final open in gameInfo.initialOpen) {
      fields[open].field = gameInfo.solution[open];
      fields[open].locked = true;
    }
  }

  static FieldInfo Function(BoardVec) _fieldInfoFactory() {
    final versionHolder = _FieldInfoVersionHolder();
    return (pos) => FieldInfo._(pos, versionHolder);
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
                return ColoredBox(color: fieldInfo.locked ? const Color(0xFF999999) : const Color(0xFFAAAAAA));
              case Field.snake:
                return const Center(
                  child: SizedBox.square(
                    dimension: 20,
                    child: ColoredBox(color: Colors.black),
                  ),
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

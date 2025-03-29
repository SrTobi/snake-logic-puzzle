import 'package:flutter/material.dart';
import 'package:flutter_mobx/flutter_mobx.dart';
import 'package:snake_logic_puzzle/models/game_info.dart';
import 'package:snake_logic_puzzle/pages/widget/game_view.dart';
import 'package:snake_logic_puzzle/utils/wrap_action.dart';
import 'package:mobx/mobx.dart';

class GamePage extends StatefulWidget {
  final GameInfo gameInfo;

  const GamePage({super.key, required this.gameInfo});

  @override
  State<GamePage> createState() => _GamePage();

  static void open(BuildContext context, {required GameInfo gameInfo}) {
    Navigator.push(
      context,
      MaterialPageRoute(
        builder: (context) => GamePage(gameInfo: gameInfo),
        settings: const RouteSettings(name: "GameView"),
      ),
    );
  }
}

class _GamePage extends State<GamePage> {
  late final GameViewModel _model;

  final Observable<bool> _setSnake = Observable(true);

  @override
  void initState() {
    super.initState();

    _model = GameViewModel(gameInfo: widget.gameInfo);
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
        appBar: AppBar(
          title: const Text('Game'),
        ),
        body: Center(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              EmptyPolicyDescriptionView(model: _model),
              const SizedBox(height: 10),
              GameView(
                model: _model,
                onClick: wrapAction1((FieldInfo field) {
                  if (!widget.gameInfo.initialOpen.contains(field.pos)) {
                    if (field.field == Field.unknown) {
                      _model.set(field.pos, _setSnake.value ? Field.snake : Field.empty);
                    } else {
                      _model.set(field.pos, Field.unknown);
                    }
                  }
                }),
              ),
              Observer(builder: (context) {
                return Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Checkbox(
                      value: _setSnake.value,
                      onChanged: wrapAction1((_) => _setSnake.value = true),
                    ),
                    Checkbox(
                      value: !_setSnake.value,
                      onChanged: wrapAction1((_) => _setSnake.value = false),
                    ),
                  ],
                );
              })
            ],
          ),
        ));
  }
}

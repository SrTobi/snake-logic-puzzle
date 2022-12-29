import 'package:flutter/cupertino.dart';
import 'package:flutter/material.dart';
import 'package:logic_snake_puzzle/pages/game_page.dart';
import 'package:logic_snake_puzzle/stores/level_store.dart';

class MainMenu extends StatelessWidget {
  const MainMenu();

  @override
  Widget build(BuildContext context) {
    final levels = LevelStore.from(context).levels;

    return Scaffold(
      appBar: AppBar(
        title: const Text('Snake Logic Puzzle'),
      ),
      body: ListView(
        children: levels
            .map(
              (level) => ListTile(
                title: const Text("Level"),
                trailing: Text("${level.width}x${level.height}"),
                onTap: () => GamePage.open(context, gameInfo: level),
              ),
            )
            .toList(growable: false),
      ),
    );
  }
}

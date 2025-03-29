import 'package:flutter/material.dart';
import 'package:logic_snake_puzzle/pages/game_page.dart';
import 'package:logic_snake_puzzle/stores/level_store.dart';

class MainMenu extends StatelessWidget {
  const MainMenu({super.key});

  @override
  Widget build(BuildContext context) {
    final levels = LevelStore.from(context).levels;

    return Scaffold(
      appBar: AppBar(
        title: const Text('Snake Logic Puzzle'),
        actions: [
          PopupMenuButton<String>(
            itemBuilder: (context) => [
              PopupMenuItem(
                child: Text("Licenses"),
                value: "show licenses",
              )
            ],
            onSelected: (_) => {
              showLicensePage(context: context)
            },
          )
        ],
      ),
      body: ListView(
        children: levels
            .map(
              (level) => ListTile(
            title: const Text("Level"),
            trailing: Text("${level.width}x${level.height} Difficulty: ${level.maxAssumptionDepth}"),
            onTap: () => GamePage.open(context, gameInfo: level),
          ),
        )
            .toList(growable: false),
      ),
    );
  }
}

import 'dart:convert';

import 'package:flutter/cupertino.dart';
import 'package:flutter/services.dart';
import 'package:logic_snake_puzzle/models/game_info.dart';
import 'package:logic_snake_puzzle/utils/developer.dart';
import 'package:provider/provider.dart';

class LevelStore {
  final List<GameInfo> levels;

  LevelStore._(this.levels);

  static LevelStore from(BuildContext context) => context.read();

  static Future<LevelStore> load() async {
    final manifestJson = await rootBundle.loadString('AssetManifest.json');
    final Map<String, dynamic> manifest = json.decode(manifestJson) as Map<String, dynamic>;
    final levelPaths = manifest.keys.where((String key) => key.startsWith('assets/levels/') && key.endsWith(".json"));

    final List<GameInfo> levels = [];

    for (final path in levelPaths) {
      dev.log("Load level $path");
      final content = await rootBundle.loadString(path);
      final gameInfo = GameInfo.loadFromJson(json.decode(content));
      levels.add(gameInfo);
    }

    return LevelStore._(List.unmodifiable(levels));
  }
}

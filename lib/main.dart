import 'dart:async';

import 'package:flutter/material.dart';
import 'package:logic_snake_puzzle/pages/main_menu.dart';
import 'package:logic_snake_puzzle/stores/level_store.dart';
import 'package:logic_snake_puzzle/utils/developer.dart';
import 'package:provider/provider.dart';

Future<void> main() async {
  runZonedGuarded<Future<void>>(
    guardedMain,
    (error, stack) {
      dev.log("Zoned async error: $error\n$stack");
    },
  );
}

Future<void> guardedMain() async {
  WidgetsFlutterBinding.ensureInitialized();

  final levelStore = LevelStore.load();
  runApp(
    MultiProvider(
      providers: [
        Provider<LevelStore>.value(value: await levelStore),
      ],
      child: const MyApp(),
    ),
  );
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  // This widget is the root of your application.
  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Snake Logic Puzzle',
      theme: ThemeData(
        primarySwatch: Colors.blue,
      ),
      home: const MainMenu(),
    );
  }
}

import 'dart:developer' as dart_developer;

class Developer {
  const Developer();

  void log(String message) {
    dart_developer.log(message);
  }
}

const Developer dev = Developer();

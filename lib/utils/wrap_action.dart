import 'package:mobx/mobx.dart';

R Function() wrapAction<R>(R Function() action) {
  return () => runInAction(action);
}

R Function(T) wrapAction1<R, T>(R Function(T) action) {
  return (value) => runInAction(() => action(value));
}

R Function(T1, T2) wrapAction2<R, T1, T2>(R Function(T1, T2) action) {
  return (a1, a2) => runInAction(() => action(a1, a2));
}

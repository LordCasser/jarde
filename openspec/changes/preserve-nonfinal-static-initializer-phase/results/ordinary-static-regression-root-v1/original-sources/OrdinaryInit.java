class OrdinaryInit {
  static int first;
  static int second;

  static { first = 4; second = twice(); }

  static int twice() { return first * 2; }
}

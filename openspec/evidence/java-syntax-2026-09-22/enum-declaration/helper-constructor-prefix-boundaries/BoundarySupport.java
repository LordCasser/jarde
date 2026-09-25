final class BoundarySupport {
  static int calls;

  static void constructorEffect() {
    calls++;
  }

  static int prefixValue(int value) {
    calls++;
    return value;
  }
}

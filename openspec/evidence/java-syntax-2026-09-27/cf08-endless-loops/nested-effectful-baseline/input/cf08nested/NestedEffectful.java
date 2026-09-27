package cf08nested;
public final class NestedEffectful {
  static int calls;
  static int cost(int x) { calls++; return x; }
  public static int pick(int[] xs) {
    int result;
    if (xs == null) {
      result = -1;
    } else {
      int i = 0;
      while (true) {
        if (i >= xs.length) { result = cost(7); break; }
        result = xs[i];
        if (result == 3) break;
        i++;
      }
      result = result + calls;
    }
    return result;
  }
}

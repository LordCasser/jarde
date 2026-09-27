package cf08effects;
public final class EffectfulExits {
  static int calls;
  static int cost(int x) { calls++; return x; }
  public static int pick(int[] xs) {
    int result;
    int i = 0;
    while (true) {
      if (i >= xs.length) { result = cost(7); break; }
      result = xs[i];
      if (result == 3) break;
      i++;
    }
    return result + calls;
  }
}

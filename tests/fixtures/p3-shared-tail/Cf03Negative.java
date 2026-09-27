public class Cf03Negative {
  public static int hits;
  public static boolean extra(boolean pre, boolean gate, int x, int y) {
    if (pre) return false;
    if (gate) {
      if (x == 0 || y == 0) return false;
    } else if (x == 0 || y == 0) return false;
    hits++;
    return true;
  }
}

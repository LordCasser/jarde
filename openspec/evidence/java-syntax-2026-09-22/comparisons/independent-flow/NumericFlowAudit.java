public class NumericFlowAudit {
  public static int floatNegated(float a, float b) {
    if (!(a >= b)) return 11;
    return 13;
  }
  public static int doubleChoice(double a, double b) {
    if (a < b) return 17;
    if (a >= b) return 19;
    return 23;
  }
  public static int longChain(long a, long b, long c) {
    if (a < b) {
      if (b < c) return 37;
      return 41;
    }
    return 43;
  }
  public static int ordered(float a, float b) {
    if (NumericEffects.left(a) > NumericEffects.right(b)) return 29;
    return 31;
  }
  public static int doubleLoop(double value, double end, double step) {
    int count = 0;
    while (value < end) {
      value += step;
      count++;
    }
    return count;
  }
}

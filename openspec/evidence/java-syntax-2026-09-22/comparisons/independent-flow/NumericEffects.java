public class NumericEffects {
  public static int trace;
  public static int throwing;
  public static float left(float value) {
    trace = trace * 10 + 1;
    if (throwing == 1) throw new IllegalStateException("left");
    return value;
  }
  public static float right(float value) {
    trace = trace * 10 + 2;
    if (throwing == 2) throw new IllegalArgumentException("right");
    return value;
  }
}

public class MultiArgs {
  public static int pick(Object x, Object y) { return 1; }
  public static int pick(String x, String y) { return 2; }
  public static int run() { return pick((Object) null, (Object) "x"); }
}

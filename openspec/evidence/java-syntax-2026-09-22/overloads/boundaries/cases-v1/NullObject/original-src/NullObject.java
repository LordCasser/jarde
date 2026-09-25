public class NullObject {
  public static int choose(Object x) { return 1; }
  public static int choose(String x) { return 2; }
  public static int run() { return choose((Object) null); }
}

public class BoxObject {
  public static int boxed(Object x) { return 5; }
  public static int boxed(Integer x) { return 6; }
  public static int run(int x) { return boxed((Object) Integer.valueOf(x)); }
}

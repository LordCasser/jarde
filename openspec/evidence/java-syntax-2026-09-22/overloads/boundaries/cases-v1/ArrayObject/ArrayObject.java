public class ArrayObject {
  public static int arr(Object x) { return 3; }
  public static int arr(String[] x) { return 4; }
  public static int run(String[] x) { return arr((Object) x); }
}

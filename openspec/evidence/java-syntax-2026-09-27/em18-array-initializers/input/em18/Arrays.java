package em18;
public class Arrays {
  public static String[] strings() { return new String[]{"1", "2", "3"}; }
  public static int[] ints(int a) { return new int[]{1, a + 1, 2}; }
  public static int[] postfix(int a) { return new int[]{1, a++, a * 2}; }
  public static int[] selfRead() {
    int[] arr = new int[3];
    arr[0] = 1;
    arr[1] = arr[0] + 1;
    arr[2] = arr[1] + 1;
    return arr;
  }
  public static int objectArg(Exception e) { return use(new Object[]{e}); }
  private static int use(Object[] values) { return values.length; }
}

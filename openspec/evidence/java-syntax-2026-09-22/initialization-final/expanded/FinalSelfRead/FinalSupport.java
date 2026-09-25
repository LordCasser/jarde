public class FinalSupport {
  public static int calls;
  public static final Object OBJECT = new Object();
  public static int next() { calls++; return calls; }
  public static boolean flag() { return Boolean.getBoolean("flag"); }
  public static Object object() { calls++; return OBJECT; }
  public static int check(Object value) { return value == OBJECT ? 1 : 0; }
}

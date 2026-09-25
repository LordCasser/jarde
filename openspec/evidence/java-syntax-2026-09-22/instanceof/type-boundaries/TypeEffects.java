public class TypeEffects {
  public static int calls;
  public static boolean fail;
  public static String value() {
    calls++;
    if (fail) throw new IllegalStateException("producer");
    return "value";
  }
}

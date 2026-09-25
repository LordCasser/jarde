public class InstanceOfTypes {
  public static boolean unrelated(String value) { return (Object) value instanceof Integer; }
  public static boolean unrelatedArray(String[] value) { return (Object) value instanceof int[]; }
  public static boolean unrelatedInterface(String value) { return (Object) value instanceof Runnable; }
  public static boolean related(String value) { return value instanceof CharSequence; }
  public static boolean called() { return (Object) TypeEffects.value() instanceof Integer; }
  public static void empty() {}
  public static boolean functional() { return ((Runnable) InstanceOfTypes::empty) instanceof Runnable; }
}

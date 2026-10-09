public final class Main {
  private static int trace;

  private static int mark(int value) {
    trace = trace * 10 + value;
    return value;
  }

  private static Byte byteValue(int value) { return Byte.valueOf((byte) value); }
  private static Short shortValue(int value) { return Short.valueOf((short) value); }
  private static Integer integerValue(int value) { return Integer.valueOf(value); }
  private static Long longValue(int value) { return Long.valueOf((long) value); }
  private static Float floatValue(int value) { return Float.valueOf((float) value); }
  private static Double doubleValue(int value) { return Double.valueOf((double) value); }

  private static Number numberValue(int value) { return Integer.valueOf(value); }
  private static Object objectValue(int value) { return String.valueOf(value); }

  public static Number[] exactNumber() { return new Number[] { numberValue(mark(1)) }; }
  public static Object[] objectElement() { return new Object[] { objectValue(mark(1)) }; }
  public static Object[] nullElement() { return new Object[] { null }; }

  private static String stringValue(int value) { return String.valueOf(value); }

  private static StringBuilder builderValue(int value) { return new StringBuilder(String.valueOf(value)); }

  private static java.util.ArrayList<Object> listValue(int value) {
    return new java.util.ArrayList<Object>(java.util.Collections.singletonList(mark(value)));
  }

  private static java.util.HashSet<Object> setValue(int value) {
    return new java.util.HashSet<Object>(java.util.Collections.singletonList(mark(value)));
  }

  private static IllegalStateException stateFailure(int value) {
    return new IllegalStateException(String.valueOf(mark(value)));
  }

  private static IllegalArgumentException argumentFailure(int value) {
    return new IllegalArgumentException(String.valueOf(mark(value)));
  }

  private static Integer[] integerArray(int value) {
    return new Integer[] { Integer.valueOf(value) };
  }

  private static Long[] longArray(int value) {
    return new Long[] { Long.valueOf((long) value) };
  }

  private static java.util.ArrayList<?>[] listArray(int value) {
    return new java.util.ArrayList<?>[] { listValue(value) };
  }

  private static java.util.HashSet<?>[] setArray(int value) {
    return new java.util.HashSet<?>[] { setValue(value) };
  }

  private static DerivedA derivedA(int value) { return new DerivedA(value); }

  private static DerivedB derivedB(int value) { return new DerivedB(value); }

  private static DerivedA[] derivedAArray(int value) { return new DerivedA[] { derivedA(value) }; }

  private static DerivedB[] derivedBArray(int value) { return new DerivedB[] { derivedB(value) }; }

  public static Number[] boxedFactory() {
    return new Number[] {
        byteValue(mark(1)), shortValue(mark(2)), integerValue(mark(3)),
        longValue(mark(4)), floatValue(mark(5)), doubleValue(mark(6))
    };
  }

  public static CharSequence[] sequenceFactory() {
    return new CharSequence[] { stringValue(mark(1)), builderValue(mark(2)) };
  }

  public static java.util.Collection<?>[] collectionFactory() {
    return new java.util.Collection<?>[] { listValue(1), setValue(2) };
  }

  public static Throwable[] throwableFactory() {
    return new Throwable[] { stateFailure(1), argumentFailure(2) };
  }

  public static Number[][] numberGridFactory() {
    return new Number[][] { integerArray(mark(1)), longArray(mark(2)) };
  }

  public static java.util.Collection<?>[][] collectionGridFactory() {
    return new java.util.Collection<?>[][] { listArray(mark(1)), setArray(mark(2)) };
  }

  public static Base[] ownTwoHopFactory() {
    return new Base[] { derivedA(mark(1)), derivedB(mark(2)) };
  }

  public static LocalInterface[] ownInterfaceFactory() {
    return new LocalInterface[] { derivedA(mark(1)), derivedB(mark(2)) };
  }

  public static Base[][] ownGridFactory() {
    return new Base[][] { derivedAArray(mark(1)), derivedBArray(mark(2)) };
  }

  private static void observe(Object[] values) {
    for (int index = 0; index < values.length; index++) {
      System.out.println(values[index] == null ? "null" : values[index].getClass().getName());
    }
    System.out.println(trace);
  }

  public static void main(String[] args) {
    trace = 0; observe(boxedFactory());
    trace = 0; observe(sequenceFactory());
    trace = 0; observe(collectionFactory());
    trace = 0; observe(throwableFactory());
    trace = 0; observe(numberGridFactory());
    trace = 0; observe(collectionGridFactory());
    trace = 0; observe(ownTwoHopFactory());
    trace = 0; observe(ownInterfaceFactory());
    trace = 0; observe(ownGridFactory());
    trace = 0; observe(exactNumber());
    trace = 0; observe(objectElement());
    trace = 0; observe(nullElement());
  }
}

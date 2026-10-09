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

  @SuppressWarnings("removal")
  public static Number[] boxedDirect() {
    return new Number[] {
        new Byte((byte) mark(1)), new Short((short) mark(2)), new Integer(mark(3)),
        new Long((long) mark(4)), new Float((float) mark(5)), new Double((double) mark(6))
    };
  }

  public static CharSequence[] sequenceDirect() {
    return new CharSequence[] {
        new String(String.valueOf(mark(1))), new StringBuilder(String.valueOf(mark(2)))
    };
  }

  public static java.util.Collection<?>[] collectionDirect() {
    return new java.util.Collection<?>[] {
        new java.util.ArrayList<Object>(java.util.Collections.singletonList(mark(1))),
        new java.util.HashSet<Object>(java.util.Collections.singletonList(mark(2)))
    };
  }

  public static Throwable[] throwableDirect() {
    return new Throwable[] {
        new IllegalStateException(String.valueOf(mark(1))),
        new IllegalArgumentException(String.valueOf(mark(2)))
    };
  }

  public static Number[][] numberGridDirect() {
    return new Number[][] {
        new Integer[] { Integer.valueOf(mark(1)) },
        new Long[] { Long.valueOf((long) mark(2)) }
    };
  }

  public static java.util.Collection<?>[][] collectionGridDirect() {
    return new java.util.Collection<?>[][] {
        new java.util.ArrayList<?>[] { listValue(mark(1)) },
        new java.util.HashSet<?>[] { setValue(mark(2)) }
    };
  }

  public static Base[] ownTwoHopDirect() {
    return new Base[] { new DerivedA(mark(1)), new DerivedB(mark(2)) };
  }

  public static LocalInterface[] ownInterfaceDirect() {
    return new LocalInterface[] { new DerivedA(mark(1)), new DerivedB(mark(2)) };
  }

  public static Base[][] ownGridDirect() {
    return new Base[][] {
        new DerivedA[] { new DerivedA(mark(1)) },
        new DerivedB[] { new DerivedB(mark(2)) }
    };
  }

  private static void observe(Object[] values) {
    for (int index = 0; index < values.length; index++) {
      System.out.println(values[index] == null ? "null" : values[index].getClass().getName());
    }
    System.out.println(trace);
  }

  public static void main(String[] args) {
    trace = 0; observe(boxedDirect());
    trace = 0; observe(sequenceDirect());
    trace = 0; observe(collectionDirect());
    trace = 0; observe(throwableDirect());
    trace = 0; observe(numberGridDirect());
    trace = 0; observe(collectionGridDirect());
    trace = 0; observe(ownTwoHopDirect());
    trace = 0; observe(ownInterfaceDirect());
    trace = 0; observe(ownGridDirect());
    trace = 0; observe(exactNumber());
    trace = 0; observe(objectElement());
    trace = 0; observe(nullElement());
  }
}

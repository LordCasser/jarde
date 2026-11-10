package p;
public class Outer {
  public static class 𐐀A<T> {
    public static int effects;
    public static int mark(int v) { effects++; return v; }
    public class Generic<V> {
      private final V value;
      public Generic(V v) { value = v; }
      public V value() { return value; }
    }
  }
}

package p;
public class 𐐀Outer {
  public static int effects;
  public static int mark(int v) { effects++; return v; }
  public class Inner {
    private final int value;
    public Inner(int v) { value = v; }
    public int value() { return value; }
  }
}

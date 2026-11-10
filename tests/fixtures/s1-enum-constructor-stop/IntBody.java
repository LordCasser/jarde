package demo;
public enum IntBody {
    PLUS(1) { public int apply(int x) { return x + 1; } },
    TIMES(2) { public int apply(int x) { return x * 2; } };
    private final int value;
    IntBody(int value) { this.value = value; }
    public int value() { return value; }
    public abstract int apply(int x);
}

package demo;

public enum Op {
    ADD { public int apply(int a, int b) { return a + b; } },
    MULTIPLY { public int apply(int a, int b) { return a * b; } };

    public abstract int apply(int a, int b);

    public String tag() {
        return name() + ":" + ordinal();
    }
}

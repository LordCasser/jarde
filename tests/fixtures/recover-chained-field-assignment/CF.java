public class CF {
    int flags = 0;
    String field = "f";
    static int sa, sb, sc;
    static String sfield = "s";
    static int[] arr = new int[2];

    // The chained field assignment: one `dup` per extra store, the value evaluated once.
    static void chain() { CF.sa = CF.sb = CF.sc = 5; }
    static void pair() { CF.sa = CF.sb = 7; }

    // The same chain whose right-hand side may not be written once per store: the lead saves it.
    static void call() { CF.sa = CF.sb = CF.sc = f(); }

    // The controls: one field assignment, and the local chain's own presentation.
    static void single() { CF.sa = 9; }
    static int chainLocal() { int x, y; x = y = 7; return x + y; }
    static int f() { return 9; }

    // The receiver copy of one member's read and write, with the operator the update rule does not
    // present.
    void enable(int bit) { flags |= 1 << bit; }
    void disable(int bit) { flags &= ~(1 << bit); }

    // The receiver copy a `String` compound assignment carries: `dup_x1` under the builder.
    CF add(String x) { field += "[" + x + "]"; return this; }

    // The static `String` compound keeps the presentation it already had (no receiver copy).
    static void sAdd(String x) { sfield += "[" + x + "]"; }

    public static void main(String[] a) {
        chain();
        pair();
        call();
        single();
        CF c = new CF();
        c.enable(0);
        c.enable(3);
        System.out.println("" + sa + "/" + sb + "/" + sc + "/" + chainLocal() + "/" + c.flags + "/" + c.add("x").add("y").field);
        c.disable(3);
        sAdd("z");
        System.out.println("" + c.flags + "/" + sfield + "/" + arr[0]);
    }
}

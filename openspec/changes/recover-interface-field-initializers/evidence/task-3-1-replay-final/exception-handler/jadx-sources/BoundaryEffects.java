package defpackage;

public final class BoundaryEffects {
    static String trace = "";
    static int value() { trace += "L"; return 9; }
    static String first() { trace += "A"; return "A"; }
    static String second() { trace += "B"; return "B"; }
    static void fail() { trace += "E"; throw new IllegalStateException("expected"); }
    static void caught() { trace += "C"; }
    private BoundaryEffects() {}
}

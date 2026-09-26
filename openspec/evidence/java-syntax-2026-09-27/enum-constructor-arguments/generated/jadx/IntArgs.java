package defpackage;

/* JADX INFO: loaded from: IntArgs.class */
public enum IntArgs {
    LITERAL(1),
    FIELD(Ints.THREE),
    EXPR(Ints.THREE + 1);

    private final int n;

    IntArgs(int n) {
        this.n = n;
    }

    public int value() {
        return this.n;
    }
}

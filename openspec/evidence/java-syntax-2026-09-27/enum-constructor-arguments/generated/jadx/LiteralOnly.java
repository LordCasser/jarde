package defpackage;

/* JADX INFO: loaded from: LiteralOnly.class */
public enum LiteralOnly {
    FIRST(7),
    NEXT(-2);

    final int n;

    LiteralOnly(int n) {
        this.n = n;
    }

    int value() {
        return this.n;
    }
}

package defpackage;

/* JADX INFO: loaded from: fixture.jar:Base.class */
class Base {
    private final int left;
    private final int right;

    Base(int i, int i2) {
        this.left = i;
        this.right = i2;
    }

    Base(int i, long j) {
        this(i, (int) j);
    }

    int sum() {
        return (this.left * 2) - this.right;
    }
}

package demo;

/* JADX INFO: loaded from: original.jar:demo/Numbers.class */
public enum Numbers {
    ZERO,
    ONE(1);

    private final int n;

    Numbers() {
        this(0);
    }

    Numbers(int n) {
        this.n = n;
    }

    public int getN() {
        return this.n;
    }
}

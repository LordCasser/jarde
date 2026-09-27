package demo;

/* JADX INFO: loaded from: original.jar:demo/Numbers.class */
public enum Numbers {
    ZERO,
    ONE(1);

    private final int n;

    Numbers() {
        this(0);
    }

    Numbers(int i) {
        this.n = i;
    }

    public int getN() {
        return this.n;
    }
}

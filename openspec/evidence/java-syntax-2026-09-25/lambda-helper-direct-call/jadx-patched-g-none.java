package defpackage;

import java.util.function.IntUnaryOperator;

/* JADX INFO: loaded from: LambdaAlias.class */
public final class LambdaAlias {
    private final int factor;

    public LambdaAlias(int i) {
        this.factor = i;
    }

    public IntUnaryOperator build(int i) {
        return i2 -> {
            return ((i2 * this.factor) + i) - 3;
        };
    }

    public int direct(int i, int i2) {
        return lambda$build$0(i, i2);
    }

    private int directHelper(int i, int i2) {
        return ((i + i2) * this.factor) + 100;
    }
}

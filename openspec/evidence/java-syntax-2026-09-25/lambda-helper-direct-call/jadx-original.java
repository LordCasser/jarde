package defpackage;

import java.util.function.IntUnaryOperator;

/* JADX INFO: loaded from: LambdaAlias.class */
public final class LambdaAlias {
    private final int factor;

    public LambdaAlias(int factor) {
        this.factor = factor;
    }

    public IntUnaryOperator build(int base) {
        return value -> {
            int scaled = value * this.factor;
            int adjusted = scaled + base;
            return adjusted - 3;
        };
    }

    public int direct(int left, int right) {
        return directHelper(left, right);
    }

    private int directHelper(int left, int right) {
        int sum = left + right;
        return (sum * this.factor) + 100;
    }
}

import java.util.function.IntUnaryOperator;

public final class LambdaAlias {
    private final int factor;

    public LambdaAlias(int factor) {
        this.factor = factor;
    }

    public IntUnaryOperator build(int base) {
        return value -> {
            int scaled = value * factor;
            int adjusted = scaled + base;
            return adjusted - 3;
        };
    }

    public int direct(int left, int right) {
        return directHelper(left, right);
    }

    private int directHelper(int left, int right) {
        int sum = left + right;
        return sum * factor + 100;
    }
}

import java.util.function.IntBinaryOperator;
import java.util.function.IntSupplier;
import java.util.function.IntUnaryOperator;

public final class LambdaFixture {
    public static IntSupplier zero() {
        return () -> 7;
    }

    public static IntUnaryOperator one() {
        return x -> x + 10;
    }

    public static IntBinaryOperator two() {
        return (left, right) -> left * 10 + right;
    }
}

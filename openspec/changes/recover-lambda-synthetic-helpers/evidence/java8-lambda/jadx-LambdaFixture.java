package defpackage;

import java.util.function.IntBinaryOperator;
import java.util.function.IntSupplier;
import java.util.function.IntUnaryOperator;

/* JADX INFO: loaded from: LambdaFixture.class */
public final class LambdaFixture {
    public static IntSupplier zero() {
        return () -> {
            return 7;
        };
    }

    public static IntUnaryOperator one() {
        return i -> {
            return i + 10;
        };
    }

    public static IntBinaryOperator two() {
        return (i, i2) -> {
            return (i * 10) + i2;
        };
    }
}

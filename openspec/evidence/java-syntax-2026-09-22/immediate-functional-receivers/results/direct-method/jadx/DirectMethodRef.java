package defpackage;

import java.util.function.IntUnaryOperator;

/* JADX INFO: loaded from: DirectMethodRef.class */
public class DirectMethodRef {
    public static int abs(int n) {
        IntUnaryOperator intUnaryOperator = Math::abs;
        return intUnaryOperator.applyAsInt(n);
    }
}

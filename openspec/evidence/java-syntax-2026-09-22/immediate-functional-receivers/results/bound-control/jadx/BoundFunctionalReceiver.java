package defpackage;

import java.util.function.IntFunction;
import java.util.function.IntUnaryOperator;

/* JADX INFO: loaded from: BoundFunctionalReceiver.class */
public class BoundFunctionalReceiver {
    public static int array(int n) {
        IntFunction<int[]> f = x$0 -> {
            return new int[x$0];
        };
        return f.apply(n).length;
    }

    public static int method(int n) {
        IntUnaryOperator f = Math::abs;
        return f.applyAsInt(n);
    }
}

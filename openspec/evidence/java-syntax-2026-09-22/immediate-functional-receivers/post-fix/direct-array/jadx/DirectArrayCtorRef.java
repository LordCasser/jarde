package defpackage;

import java.util.function.IntFunction;

/* JADX INFO: loaded from: DirectArrayCtorRef.class */
public class DirectArrayCtorRef {
    public static int[] make(int n) {
        IntFunction intFunction = x$0 -> {
            return new int[x$0];
        };
        return (int[]) intFunction.apply(n);
    }
}

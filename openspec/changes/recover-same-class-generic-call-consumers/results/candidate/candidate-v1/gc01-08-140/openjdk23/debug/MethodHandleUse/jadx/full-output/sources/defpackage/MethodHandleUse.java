package defpackage;

import java.util.function.Function;

/* JADX INFO: loaded from: MethodHandleUse.jar:MethodHandleUse.class */
public class MethodHandleUse<T> {
    public T identity(T x) {
        return x;
    }

    public T relay(T x) {
        Function<T, T> f = this::identity;
        return f.apply(x);
    }
}

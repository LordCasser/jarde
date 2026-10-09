package defpackage;

import java.lang.Number;

/* JADX INFO: loaded from: NumberBoundRelay.jar:NumberBoundRelay.class */
public class NumberBoundRelay<T extends Number> {
    public T identity(T x) {
        return x;
    }

    public T relay(T t) {
        return (T) identity(t);
    }
}

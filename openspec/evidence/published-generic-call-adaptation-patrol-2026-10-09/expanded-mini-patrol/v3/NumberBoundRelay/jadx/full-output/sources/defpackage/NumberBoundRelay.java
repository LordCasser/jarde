package defpackage;

import java.lang.Number;

/* JADX INFO: loaded from: NumberBoundRelay.jar:NumberBoundRelay.class */
public class NumberBoundRelay<T extends Number> {
    public T id(T x) {
        return x;
    }

    public T relay(T t) {
        return (T) id(t);
    }
}

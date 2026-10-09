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

    public static void main(String[] a) {
        NumberBoundRelay<Integer> c = new NumberBoundRelay<>();
        System.out.println("behavior.marker=" + (c.relay(17) == 17));
    }
}

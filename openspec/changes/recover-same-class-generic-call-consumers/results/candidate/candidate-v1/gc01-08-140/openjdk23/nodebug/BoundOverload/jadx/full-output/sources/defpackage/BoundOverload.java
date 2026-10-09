package defpackage;

import java.lang.Comparable;
import java.lang.Number;

/* JADX INFO: loaded from: BoundOverload.jar:BoundOverload.class */
public class BoundOverload<T extends Number & Comparable<T>> {
    public void pick(Number number) {
        System.out.println("number");
    }

    public void pick(Comparable<T> comparable) {
        System.out.println("comparable");
    }

    public void relay(T t) {
        pick(t);
    }
}

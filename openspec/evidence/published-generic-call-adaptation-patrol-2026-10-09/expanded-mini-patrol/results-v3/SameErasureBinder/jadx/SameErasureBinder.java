package defpackage;

import java.lang.Number;
import java.lang.Runnable;

/* JADX INFO: loaded from: SameErasureBinder.jar:SameErasureBinder.class */
public class SameErasureBinder<T extends Number & Runnable> {
    public <U extends Number & Runnable> U sink(U x) {
        return x;
    }

    public T relay(T t) {
        return (T) sink(t);
    }
}

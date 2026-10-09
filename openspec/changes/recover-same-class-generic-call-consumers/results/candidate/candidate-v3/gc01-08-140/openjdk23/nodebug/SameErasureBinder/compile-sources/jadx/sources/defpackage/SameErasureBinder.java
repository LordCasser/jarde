package defpackage;

import java.lang.Number;

/* JADX INFO: loaded from: SameErasureBinder.jar:SameErasureBinder.class */
public class SameErasureBinder<T extends Number> {
    public int calls;

    public <U extends Number & Runnable> U sink(U u) {
        this.calls++;
        return u;
    }

    public T independent(T t) {
        return t;
    }

    public void useNull() {
        sink(null);
    }
}

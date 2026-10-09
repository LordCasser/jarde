package defpackage;

import java.lang.Number;

/* JADX INFO: loaded from: SameErasureBinder.jar:SameErasureBinder.class */
public class SameErasureBinder<T extends Number> {
    public int calls;

    public <U extends Number & Runnable> U sink(U x) {
        this.calls++;
        return x;
    }

    public T independent(T x) {
        return x;
    }

    public void useNull() {
        sink(null);
    }
}

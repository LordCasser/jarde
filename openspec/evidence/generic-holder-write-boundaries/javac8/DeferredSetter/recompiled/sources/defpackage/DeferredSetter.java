package defpackage;

/* JADX INFO: loaded from: DeferredSetter.jar:DeferredSetter.class */
public class DeferredSetter<T> {
    public T v;

    public void put(T t) {
        sink(t);
        this.v = t;
    }

    private void sink(T t) {
    }
}

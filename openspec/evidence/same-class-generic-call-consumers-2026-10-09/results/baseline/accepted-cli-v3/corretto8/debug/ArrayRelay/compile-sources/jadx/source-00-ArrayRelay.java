package defpackage;

/* JADX INFO: loaded from: ArrayRelay.jar:ArrayRelay.class */
public class ArrayRelay<T> {
    public T[] identity(T[] x) {
        return x;
    }

    public T[] relay(T[] x) {
        return identity(x);
    }
}

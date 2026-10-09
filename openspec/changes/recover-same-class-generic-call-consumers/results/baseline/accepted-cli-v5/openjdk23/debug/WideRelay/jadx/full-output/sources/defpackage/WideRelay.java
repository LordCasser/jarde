package defpackage;

/* JADX INFO: loaded from: WideRelay.jar:WideRelay.class */
public class WideRelay<T> {
    public T identity(T x) {
        return x;
    }

    public T relay(long before, T x, double middle, T y) {
        return identity(y);
    }
}

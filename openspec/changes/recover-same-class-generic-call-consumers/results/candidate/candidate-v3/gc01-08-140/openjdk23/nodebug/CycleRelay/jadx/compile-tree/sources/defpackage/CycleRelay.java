package defpackage;

/* JADX INFO: loaded from: CycleRelay.jar:CycleRelay.class */
public class CycleRelay<T> {
    public T left(T t, boolean z) {
        return z ? right(t, false) : t;
    }

    public T right(T t, boolean z) {
        return z ? left(t, false) : t;
    }
}

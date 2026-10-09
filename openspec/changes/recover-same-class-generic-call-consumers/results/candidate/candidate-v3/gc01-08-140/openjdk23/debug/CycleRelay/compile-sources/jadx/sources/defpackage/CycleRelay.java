package defpackage;

/* JADX INFO: loaded from: CycleRelay.jar:CycleRelay.class */
public class CycleRelay<T> {
    public T left(T x, boolean again) {
        return again ? right(x, false) : x;
    }

    public T right(T x, boolean again) {
        return again ? left(x, false) : x;
    }
}

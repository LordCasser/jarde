package defpackage;

/* JADX INFO: loaded from: PhiHold.jar:PhiHold.class */
public class PhiHold<T> {
    public T v;

    public PhiHold(T v, boolean choose) {
        this.v = choose ? v : null;
    }
}

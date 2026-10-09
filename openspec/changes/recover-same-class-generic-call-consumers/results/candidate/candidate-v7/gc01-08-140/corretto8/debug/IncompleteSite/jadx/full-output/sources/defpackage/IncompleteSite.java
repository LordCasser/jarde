package defpackage;

/* JADX INFO: loaded from: IncompleteSite.jar:IncompleteSite.class */
public class IncompleteSite<T> {
    public T identity(T x) {
        return x;
    }

    public T relay(T x, boolean use) {
        return use ? identity(x) : x;
    }
}

package defpackage;

/* JADX INFO: loaded from: IncompleteSite.jar:IncompleteSite.class */
public class IncompleteSite<T> {
    public T identity(T t) {
        return t;
    }

    public T relay(T t, boolean z) {
        return z ? identity(t) : t;
    }
}

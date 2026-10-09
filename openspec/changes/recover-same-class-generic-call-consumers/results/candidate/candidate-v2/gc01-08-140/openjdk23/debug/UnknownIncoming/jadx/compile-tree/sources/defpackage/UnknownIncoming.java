package defpackage;

/* JADX INFO: loaded from: UnknownIncoming.jar:UnknownIncoming.class */
public class UnknownIncoming<T> {
    public T identity(T x) {
        return x;
    }

    public T safe(T x) {
        return identity(x);
    }

    public T untouched(T x) {
        return x;
    }

    /* JADX WARN: Multi-variable type inference failed */
    public T unsafe(Object obj) {
        return identity(obj);
    }
}

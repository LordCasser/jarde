package defpackage;

/* JADX INFO: loaded from: UnknownIncoming.jar:UnknownIncoming.class */
public class UnknownIncoming<T> {
    public T identity(T t) {
        return t;
    }

    public T safe(T t) {
        return identity(t);
    }

    public T untouched(T t) {
        return t;
    }

    /* JADX WARN: Multi-variable type inference failed */
    public T unsafe(Object obj) {
        return identity(obj);
    }
}

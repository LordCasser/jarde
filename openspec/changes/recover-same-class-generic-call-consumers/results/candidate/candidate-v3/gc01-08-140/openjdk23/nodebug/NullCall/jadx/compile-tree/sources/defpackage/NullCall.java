package defpackage;

/* JADX INFO: loaded from: NullCall.jar:NullCall.class */
public class NullCall<T> {
    public T identity(T t) {
        return t;
    }

    public T relay() {
        return identity(null);
    }
}

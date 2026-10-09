package defpackage;

/* JADX INFO: loaded from: CallRelay.jar:CallRelay.class */
public class CallRelay<T> {
    public T identity(T t) {
        return t;
    }

    public T relay(T t) {
        return identity(t);
    }
}

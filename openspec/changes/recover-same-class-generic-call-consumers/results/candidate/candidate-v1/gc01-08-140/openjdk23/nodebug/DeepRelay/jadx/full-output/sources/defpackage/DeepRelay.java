package defpackage;

/* JADX INFO: loaded from: DeepRelay.jar:DeepRelay.class */
public class DeepRelay<T> {
    public T relay0(T t) {
        return relay1(t);
    }

    public T relay1(T t) {
        return relay2(t);
    }

    public T relay2(T t) {
        return identity(t);
    }

    public T identity(T t) {
        return t;
    }
}

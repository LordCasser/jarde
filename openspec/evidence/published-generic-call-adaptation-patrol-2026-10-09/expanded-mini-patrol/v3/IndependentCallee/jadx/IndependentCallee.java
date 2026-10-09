package defpackage;

/* JADX INFO: loaded from: IndependentCallee.jar:IndependentCallee.class */
public class IndependentCallee<T> {
    public <U> U id(U x) {
        return x;
    }

    public T relay(T t) {
        return (T) id(t);
    }
}

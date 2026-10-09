package defpackage;

/* JADX INFO: loaded from: IndependentCallee.jar:IndependentCallee.class */
public class IndependentCallee<T> {
    public <U> U identity(U u) {
        return u;
    }

    public T relay(T t) {
        return (T) identity(t);
    }
}

package defpackage;

/* JADX INFO: loaded from: MultiUseResult.jar:MultiUseResult.class */
public class MultiUseResult<T> {
    public Object observed;

    public T identity(T t) {
        return t;
    }

    public void observe(Object obj) {
        this.observed = obj;
    }

    public T relay(T t) {
        T tIdentity = identity(t);
        observe(tIdentity);
        return tIdentity;
    }
}

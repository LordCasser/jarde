package defpackage;

/* JADX INFO: loaded from: MultiUseResult.jar:MultiUseResult.class */
public class MultiUseResult<T> {
    public Object observed;

    public T identity(T x) {
        return x;
    }

    public void observe(Object x) {
        this.observed = x;
    }

    public T relay(T x) {
        T result = identity(x);
        observe(result);
        return result;
    }
}

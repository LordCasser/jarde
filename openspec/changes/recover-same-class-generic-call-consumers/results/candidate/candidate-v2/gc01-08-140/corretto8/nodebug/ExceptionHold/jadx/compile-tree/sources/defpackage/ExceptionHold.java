package defpackage;

/* JADX INFO: loaded from: ExceptionHold.jar:ExceptionHold.class */
public class ExceptionHold<T> {
    public T v;

    public ExceptionHold(T t) {
        try {
            this.v = maybe(t);
        } catch (RuntimeException e) {
            this.v = null;
        }
    }

    private T maybe(T t) {
        return t;
    }
}

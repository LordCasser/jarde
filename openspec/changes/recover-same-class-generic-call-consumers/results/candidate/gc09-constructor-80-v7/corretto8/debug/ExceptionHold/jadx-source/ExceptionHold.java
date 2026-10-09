/* JADX INFO: loaded from: ExceptionHold.jar:ExceptionHold.class */
public class ExceptionHold<T> {
    public T v;

    public ExceptionHold(T v) {
        try {
            this.v = maybe(v);
        } catch (RuntimeException e) {
            this.v = null;
        }
    }

    private T maybe(T x) {
        return x;
    }
}

package defpackage;

/* JADX INFO: loaded from: MultiParam.jar:MultiParam.class */
public class MultiParam<T> {
    public T first(T x, T y) {
        return x;
    }

    public T relay(T x, T y) {
        return first(x, y);
    }

    /* JADX WARN: Multi-variable type inference failed */
    public static void main(String[] a) {
        Object m = new Object();
        System.out.println("behavior.marker=" + (new MultiParam().relay(m, m) == m));
    }
}

package defpackage;

/* JADX INFO: loaded from: ArrayRelay.jar:ArrayRelay.class */
public class ArrayRelay<T> {
    public T[] id(T[] x) {
        return x;
    }

    public T[] relay(T[] x) {
        return id(x);
    }

    public static void main(String[] a) {
        Object[] m = {new Object()};
        ArrayRelay<Object> c = new ArrayRelay<>();
        System.out.println("behavior.marker=" + (c.relay(m) == m));
    }
}

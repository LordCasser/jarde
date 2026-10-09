package defpackage;

/* JADX INFO: loaded from: IndependentCallee.jar:IndependentCallee.class */
public class IndependentCallee<T> {
    public <U> U id(U x) {
        return x;
    }

    public T relay(T t) {
        return (T) id(t);
    }

    /* JADX WARN: Multi-variable type inference failed */
    public static void main(String[] a) {
        Object m = new Object();
        System.out.println("behavior.marker=" + (new IndependentCallee().relay(m) == m));
    }
}

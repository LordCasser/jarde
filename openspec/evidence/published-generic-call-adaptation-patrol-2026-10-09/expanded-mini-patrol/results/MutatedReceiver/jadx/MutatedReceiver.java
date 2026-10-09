package defpackage;

/* JADX INFO: loaded from: MutatedReceiver.jar:MutatedReceiver.class */
public class MutatedReceiver<T> {
    public T relay(T t) {
        new Box();
        return (T) new Box().id(t);
    }

    /* JADX WARN: Multi-variable type inference failed */
    public static void main(String[] a) {
        Object m = new Object();
        System.out.println("behavior.marker=" + (new MutatedReceiver().relay(m) == m));
    }
}

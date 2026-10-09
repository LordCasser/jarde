package defpackage;

/* JADX INFO: loaded from: RawReceiver.jar:RawReceiver.class */
public class RawReceiver<T> {
    public Box box = new Box();

    public T relay(T t) {
        return (T) this.box.id(t);
    }

    public static void main(String[] a) {
        Object m = new Object();
        RawReceiver<Object> c = new RawReceiver<>();
        System.out.println("behavior.marker=" + (c.relay(m) == m));
    }
}

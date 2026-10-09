package defpackage;

/* JADX INFO: loaded from: VoidDirect.jar:VoidDirect.class */
public class VoidDirect<T> {
    Object seen;

    public void sink(T x) {
        this.seen = x;
    }

    public void relay(T x) {
        sink(x);
    }

    public static void main(String[] a) throws Exception {
        Object m = new Object();
        VoidDirect<Object> c = new VoidDirect<>();
        c.relay(m);
        System.out.println("behavior.marker=" + (c.seen == m));
    }
}

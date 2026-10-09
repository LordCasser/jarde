package defpackage;

/* JADX INFO: loaded from: VoidDirect.jar:VoidDirect.class */
public class VoidDirect<T> {
    public Object seen;
    public int calls;

    public void sink(T x) {
        this.seen = x;
        this.calls++;
    }

    public void relay(T x) {
        sink(x);
    }
}

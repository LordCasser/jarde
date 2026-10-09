package defpackage;

/* JADX INFO: loaded from: VoidDirect.jar:VoidDirect.class */
public class VoidDirect<T> {
    public Object seen;
    public int calls;

    public void sink(T t) {
        this.seen = t;
        this.calls++;
    }

    public void relay(T t) {
        sink(t);
    }
}

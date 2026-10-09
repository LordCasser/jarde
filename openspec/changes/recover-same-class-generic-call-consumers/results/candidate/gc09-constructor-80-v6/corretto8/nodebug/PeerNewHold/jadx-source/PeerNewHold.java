/* JADX INFO: loaded from: PeerNewHold.jar:PeerNewHold.class */
public class PeerNewHold<T> {
    public T v;

    public PeerNewHold(T t) {
        this.v = t;
        new PeerNewHold(t, true);
    }

    public PeerNewHold(T t, boolean z) {
        this.v = t;
    }
}

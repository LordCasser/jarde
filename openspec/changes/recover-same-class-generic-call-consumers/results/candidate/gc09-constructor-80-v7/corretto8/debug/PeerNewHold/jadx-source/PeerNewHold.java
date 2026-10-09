/* JADX INFO: loaded from: PeerNewHold.jar:PeerNewHold.class */
public class PeerNewHold<T> {
    public T v;

    public PeerNewHold(T v) {
        this.v = v;
        new PeerNewHold(v, true);
    }

    public PeerNewHold(T v, boolean peer) {
        this.v = v;
    }
}

public class PeerNewHold<T> { public T v; public PeerNewHold(T v) { this.v = v; PeerNewHold<T> peer = new PeerNewHold<T>(v, true); } public PeerNewHold(T v, boolean peer) { this.v = v; } }

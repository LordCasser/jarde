package defpackage;
public final class BridgeRunner {
    public static void main(String[] args) {
        BridgeProbe probe = new BridgeProbe();
        BridgeApi raw = probe;
        System.out.println(probe.get() + "|" + raw.get() + "|" + BridgeProbe.observe());
    }
}

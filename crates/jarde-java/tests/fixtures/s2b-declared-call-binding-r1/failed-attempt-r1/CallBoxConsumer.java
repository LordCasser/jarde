package flow;

import java.util.List;

public class CallBoxConsumer {
    public static Object rawGet(CallBox box) { return box.get(); }
    public static List<String> rawFixed(CallBox box) { return box.fixed(); }
    public static void rawSet(CallBox box, String value) { box.set(value); }
    public static Object rawInfer(CallBox box, Object value) { return box.infer(value); }
    public static void twoArgSet(CallBox box, int left, int right) { box.set(left, right); }
    public static void bridgeEcho(BridgeChild box, String value) { box.echo(value); }
    public static void inheritedKeep(KeepChild box, String value) { box.keep(value); }
    public static void typedCollision(CallBox<String> box, String value) { box.set(value); }
}

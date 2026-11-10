package flow;

import java.util.List;

public class CallBoxConsumer {
    public static Object rawGet(CallBox box) { return box.get(); }
    public static List<String> rawFixed(CallBox box) { return box.fixed(); }
    public static void collisionSources(CallBox raw, CallBox<String> typed, String value) {
        raw.set(value);
    }
    public static Object typedInfer(CallBox<String> box, String value) { return box.infer(value); }
    public static void varargRun(VarargBox box, String value) { box.run(value); }
    public static void bridgeEcho(BridgeChild box, String value) { box.echo(value); }
    public static void inheritedKeep(KeepChild box, String value) { box.keep(value); }
}

package defpackage;

/* JADX INFO: loaded from: input.jar:BridgeProbe.class */
public final class BridgeProbe implements BridgeApi<String> {
    /* JADX WARN: Can't rename method to resolve collision */
    @Override // defpackage.BridgeApi
    public String get() {
        return "value";
    }

    public static String observe() {
        BridgeApi<String> api = new BridgeProbe();
        return api.get();
    }
}

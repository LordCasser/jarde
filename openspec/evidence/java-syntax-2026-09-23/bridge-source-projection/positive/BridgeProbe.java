interface BridgeApi<T> {
    T get();
}

public final class BridgeProbe implements BridgeApi<String> {
    @Override
    public String get() {
        return "value";
    }

    public static String observe() {
        BridgeApi<String> api = new BridgeProbe();
        return api.get();
    }
}

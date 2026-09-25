public final class FakeBridge {
    private static int calls;

    public String get() {
        return "value";
    }

    public Object geh() {
        calls++;
        return get();
    }

    public static int count() {
        return calls;
    }
}

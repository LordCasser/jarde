public final class Target {
    static {
        Trace.value += "C";
    }

    public Target(int value) {
        Trace.value += "T";
    }
}

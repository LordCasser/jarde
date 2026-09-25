public final class NonInvokeQualifierProbe {
    public static int call(Child arg) {
        return arg.ping();
    }
}

public final class CapturedType {
    private static String pick(Number value) { return "number"; }
    private static String pick(Object value) { return "object"; }
    public static String run(boolean flag) {
        Number value;
        if (flag) value = Integer.valueOf(3);
        else value = Double.valueOf(4.0);
        java.util.function.Supplier<String> callback = () -> pick(value);
        return callback.get();
    }
}

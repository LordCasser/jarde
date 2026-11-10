import java.util.function.Supplier;

public final class CaptureUpcasts {
    private CaptureUpcasts() {}

    private static String pick(Number value) { return "number"; }
    private static String pick(Object value) { return "object"; }
    private static String kind(Object value) { return "object"; }
    private static String kind(String value) { return "string"; }

    public static String one(int input) {
        Number n = Integer.valueOf(input);
        Supplier<String> action = () -> pick(n);
        return action.get();
    }

    public static String two() {
        Object value = "literal";
        Supplier<String> action = () -> kind(value);
        return action.get();
    }
}

/** The Java 8 output level: `+` is a `StringBuilder` chain, and a lambda is the only `invokedynamic`. */
public class ConcatJava8 {
    public static String mixed(int count, Object value) {
        return "count=" + count + " value=" + value + ".";
    }

    public static Runnable lambda() {
        return () -> {};
    }
}

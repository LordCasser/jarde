package shadow;

public class ShadowBounded<T extends Number> {
    public static <T extends CharSequence> T echo(T value) {
        return value;
    }
}

package shadow;

public class ShadowPlain<T> {
    public static <T> T echo(T value) {
        return value;
    }
}

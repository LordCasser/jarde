package probe;
public class StaticThrows {
    public static <T, X extends Exception> T echo(T value) throws X { return value; }
}

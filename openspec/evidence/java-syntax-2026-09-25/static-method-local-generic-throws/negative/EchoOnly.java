package evidence;

public final class EchoOnly {
    private EchoOnly() {}

    public static <T, X extends Exception> T echo(T value) throws X {
        return value;
    }
}

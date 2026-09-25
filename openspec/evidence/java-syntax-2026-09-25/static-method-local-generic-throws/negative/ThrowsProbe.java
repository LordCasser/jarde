package evidence;

import java.io.IOException;

public final class ThrowsProbe {
    private ThrowsProbe() {}

    public static <T, X extends Exception> T echo(T value) throws X {
        return value;
    }

    public static <T, X extends IOException> T ioEcho(T value) throws X {
        return value;
    }

    public static <T, X extends Exception> T sideEffect(T value) throws X {
        System.nanoTime();
        return value;
    }

    public static <T, X extends Exception> T wrapped(T value) throws X {
        return echo(value);
    }

    public static Object runner(Object value) throws Exception {
        return echo(value);
    }

    public static void main(String[] args) throws Exception {
        if (echo("ok") != "ok" || runner("same-class") != "same-class") {
            throw new AssertionError();
        }
        System.out.println("verified");
    }
}

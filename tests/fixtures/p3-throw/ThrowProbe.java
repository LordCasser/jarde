import java.io.IOException;

public final class ThrowProbe {
    public static void nullValue() {
        throw null;
    }

    public static void parameter(RuntimeException problem) {
        throw problem;
    }

    public static void allocation() {
        throw new IllegalStateException("fresh");
    }

    public static void call() {
        throw ThrowEffects.problem();
    }

    public static void cast(Object problem) {
        throw (RuntimeException) problem;
    }

    public static void conditional(boolean first, RuntimeException a, RuntimeException b) {
        if (first) {
            throw a;
        }
        throw b;
    }

    public static RuntimeException namedCatch(RuntimeException problem) {
        try {
            throw problem;
        } catch (RuntimeException caught) {
            return caught;
        }
    }

    public static void checked(IOException problem) throws IOException {
        throw problem;
    }
}

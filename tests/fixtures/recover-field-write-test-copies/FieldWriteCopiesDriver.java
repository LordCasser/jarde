import java.lang.reflect.Field;
import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.net.URL;
import java.net.URLClassLoader;
import java.nio.file.Path;
import java.nio.file.Paths;

public final class FieldWriteCopiesDriver {
    private static void require(boolean condition, String message) {
        if (!condition) {
            throw new AssertionError(message);
        }
    }

    private static Object call(Method method, Object receiver, Object... args) throws Throwable {
        try {
            return method.invoke(receiver, args);
        } catch (InvocationTargetException failure) {
            throw failure.getCause();
        }
    }

    private static Field field(Class<?> type, String name) throws Exception {
        Field field = type.getField(name);
        field.setAccessible(true);
        return field;
    }

    private static Method method(Class<?> type, String name, Class<?>... parameters) throws Exception {
        Method method = type.getMethod(name, parameters);
        method.setAccessible(true);
        return method;
    }

    private static Object instance(Class<?> type) throws Exception {
        return type.getConstructor().newInstance();
    }

    private static void printState(String label, Object receiver, Field z, Field calls,
                                   Field ledger, Field lastValue) throws Exception {
        System.out.println(label + " z=" + z.getInt(receiver)
                + " calls=" + calls.getInt(receiver)
                + " ledger=" + ledger.getLong(receiver)
                + " lastValue=" + lastValue.getInt(receiver));
    }

    public static void main(String[] args) throws Throwable {
        if (args.length != 2) {
            throw new IllegalArgumentException("expected binary class name and exact target class root");
        }
        Path expectedRoot = Paths.get(args[1]).toRealPath();
        URL targetUrl = expectedRoot.toUri().toURL();
        try (URLClassLoader loader = new URLClassLoader(new URL[] { targetUrl }, null)) {
            Class<?> type = Class.forName(args[0], true, loader);
            Path actualRoot = Paths.get(type.getProtectionDomain().getCodeSource().getLocation().toURI()).toRealPath();
            if (!actualRoot.equals(expectedRoot)) {
                throw new AssertionError("target CodeSource mismatch: " + actualRoot + " != " + expectedRoot);
            }

            Field z = field(type, "z");
            Field calls = field(type, "calls");
            Field ledger = field(type, "ledger");
            Field throwAt = field(type, "throwAt");
            Field marker = field(type, "marker");
            Field lastValue = field(type, "lastValue");
            Method assign = method(type, "assignAndCompare", int.class);
            Method loop = method(type, "statefulWhile", int.class);

            Object direct = instance(type);
            z.setInt(direct, 41);
            Object directFirst = call(assign, direct, 4);
            System.out.println("direct-first result=" + directFirst);
            printState("direct-first-state", direct, z, calls, ledger, lastValue);
            Object directSecond = call(assign, direct, -3);
            System.out.println("direct-same-instance result=" + directSecond);
            printState("direct-same-instance-state", direct, z, calls, ledger, lastValue);

            int oldZ = z.getInt(direct);
            int nextCall = calls.getInt(direct) + 1;
            Object expectedMarker = marker.get(direct);
            throwAt.setInt(direct, nextCall);
            boolean sameMarker = false;
            try {
                call(assign, direct, 8);
            } catch (Throwable failure) {
                sameMarker = failure == expectedMarker;
            }
            require(sameMarker, "direct assignment must rethrow the exact stored marker");
            require(z.getInt(direct) == oldZ, "failed RHS must leave the prior field value intact");
            System.out.println("direct-throw sameMarker=" + sameMarker + " oldZ=" + oldZ
                    + " actualZ=" + z.getInt(direct));
            printState("direct-throw-state", direct, z, calls, ledger, lastValue);

            Object looping = instance(type);
            Object loopResult = call(loop, looping, 0);
            System.out.println("while-complete result=" + loopResult);
            printState("while-complete-state", looping, z, calls, ledger, lastValue);

            Object loopThrowing = instance(type);
            z.setInt(loopThrowing, 91);
            Object loopMarker = marker.get(loopThrowing);
            throwAt.setInt(loopThrowing, 2);
            boolean loopSameMarker = false;
            try {
                call(loop, loopThrowing, 0);
            } catch (Throwable failure) {
                loopSameMarker = failure == loopMarker;
            }
            require(loopSameMarker, "loop assignment must rethrow the exact stored marker");
            require(z.getInt(loopThrowing) == 1, "second loop RHS failure must retain first completed write");
            System.out.println("while-throw sameMarker=" + loopSameMarker + " z=" + z.getInt(loopThrowing));
            printState("while-throw-state", loopThrowing, z, calls, ledger, lastValue);
        }
    }
}

import java.lang.reflect.Field;
import java.lang.reflect.Method;
import java.net.URL;
import java.net.URLClassLoader;
import java.nio.file.Path;
import java.nio.file.Paths;

public final class StatementfulLoopBoundaryDriver {
    private StatementfulLoopBoundaryDriver() {}

    public static void main(String[] args) throws Exception {
        if (args.length != 1) {
            throw new IllegalArgumentException("expected target class root");
        }
        Path expectedRoot = Paths.get(args[0]).toRealPath();
        try (URLClassLoader loader = new URLClassLoader(new URL[] { expectedRoot.toUri().toURL() }, null)) {
            Class<?> target = Class.forName("StatementfulLoopBoundary", true, loader);
            Path actualRoot = Paths.get(target.getProtectionDomain().getCodeSource().getLocation().toURI()).toRealPath();
            if (!expectedRoot.equals(actualRoot)) {
                throw new AssertionError("target loaded from " + actualRoot + ", expected " + expectedRoot);
            }
            Field observed = target.getField("observed");
            Field calls = target.getField("calls");
            Method extraExit = target.getMethod("extraBodyExit", int.class);
            Method multipleLatches = target.getMethod("multipleLatches", int.class);

            Object exitReceiver = target.getConstructor().newInstance();
            int exitResult = ((Integer) extraExit.invoke(exitReceiver, 10)).intValue();
            System.out.println("extraBodyExit result=" + exitResult
                    + " observed=" + observed.getInt(exitReceiver)
                    + " calls=" + calls.getInt(exitReceiver));

            Object latchReceiver = target.getConstructor().newInstance();
            int latchResult = ((Integer) multipleLatches.invoke(latchReceiver, 5)).intValue();
            System.out.println("multipleLatches result=" + latchResult
                    + " observed=" + observed.getInt(latchReceiver)
                    + " calls=" + calls.getInt(latchReceiver));
        }
    }
}

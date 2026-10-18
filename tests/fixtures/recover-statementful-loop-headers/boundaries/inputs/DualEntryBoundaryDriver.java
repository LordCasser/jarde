import java.lang.reflect.Method;
import java.net.URL;
import java.net.URLClassLoader;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.Arrays;

public final class DualEntryBoundaryDriver {
    private DualEntryBoundaryDriver() {}

    public static void main(String[] args) throws Exception {
        if (args.length != 1) {
            throw new IllegalArgumentException("expected target class root");
        }
        Path expectedRoot = Paths.get(args[0]).toRealPath();
        URL targetUrl = expectedRoot.toUri().toURL();
        try (URLClassLoader loader = new URLClassLoader(new URL[] {targetUrl}, null)) {
            Class<?> target = Class.forName("StatementfulDualEntry", true, loader);
            Path actualRoot = Paths.get(
                    target.getProtectionDomain().getCodeSource().getLocation().toURI()).toRealPath();
            if (!expectedRoot.equals(actualRoot)) {
                throw new AssertionError("target loaded from " + actualRoot + ", expected " + expectedRoot);
            }
            Method run = target.getDeclaredMethod("run", int.class, int.class);
            Method tick = target.getDeclaredMethod("tick", int.class);
            if (!java.lang.reflect.Modifier.isStatic(run.getModifiers())
                    || !java.lang.reflect.Modifier.isStatic(tick.getModifiers())) {
                throw new AssertionError("expected the two static classfile methods");
            }
            String[] names = Arrays.stream(target.getDeclaredMethods())
                    .map(Method::getName)
                    .sorted()
                    .toArray(String[]::new);
            if (!Arrays.equals(names, new String[] {"run", "tick"})) {
                throw new AssertionError("unexpected method inventory: " + Arrays.toString(names));
            }
            System.out.println("verified StatementfulDualEntry methods=" + Arrays.toString(names));
        }
    }
}

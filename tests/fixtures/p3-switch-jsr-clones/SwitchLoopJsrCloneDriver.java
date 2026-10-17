import java.lang.reflect.Method;
import java.net.URL;
import java.net.URLClassLoader;
import java.nio.file.Path;
import java.nio.file.Paths;

public final class SwitchLoopJsrCloneDriver {
    private SwitchLoopJsrCloneDriver() {}

    public static void main(String[] args) throws Exception {
        if (args.length != 1) {
            throw new IllegalArgumentException("expected target class root");
        }
        Path expectedRoot = Paths.get(args[0]).toRealPath();
        URL targetUrl = expectedRoot.toUri().toURL();
        try (URLClassLoader loader = new URLClassLoader(new URL[] {targetUrl}, null)) {
            Class<?> target = Class.forName("SwitchLoopJsrClone", true, loader);
            Path actualRoot = Paths.get(
                    target.getProtectionDomain().getCodeSource().getLocation().toURI()).toRealPath();
            if (!expectedRoot.equals(actualRoot)) {
                throw new AssertionError("target loaded from " + actualRoot + ", expected " + expectedRoot);
            }
            Method run = target.getDeclaredMethod("run", int.class);
            run.setAccessible(true);
            int zero = ((Integer) run.invoke(null, 0)).intValue();
            int three = ((Integer) run.invoke(null, 3)).intValue();
            if (zero != 0 || three != 12) {
                throw new AssertionError("unexpected results: " + zero + ", " + three);
            }
            System.out.println("run(0)=" + zero);
            System.out.println("run(3)=" + three);
        }
    }
}

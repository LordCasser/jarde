import java.lang.reflect.Constructor;
import java.lang.reflect.Field;
import java.lang.reflect.Method;
import java.lang.reflect.Modifier;
import java.net.URL;
import java.net.URLClassLoader;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.Arrays;

public final class ExtraStackCopyDriver {
    private ExtraStackCopyDriver() {}

    public static void main(String[] args) throws Exception {
        if (args.length != 1) throw new IllegalArgumentException("expected target class root");
        Path expectedRoot = Paths.get(args[0]).toRealPath();
        try (URLClassLoader loader = new URLClassLoader(new URL[] {expectedRoot.toUri().toURL()}, null)) {
            Class<?> target = Class.forName("FieldWriteCopies", true, loader);
            Path actualRoot = Paths.get(target.getProtectionDomain().getCodeSource().getLocation().toURI()).toRealPath();
            if (!expectedRoot.equals(actualRoot)) throw new AssertionError("CodeSource mismatch: " + actualRoot);
            String[] names = Arrays.stream(target.getDeclaredMethods()).map(Method::getName).sorted().toArray(String[]::new);
            if (!Arrays.equals(names, new String[] {"assignAndCompare", "next", "statefulWhile"})) {
                throw new AssertionError("method inventory changed: " + Arrays.toString(names));
            }
            Constructor<?> constructor = target.getDeclaredConstructor();
            if (!Modifier.isPublic(constructor.getModifiers())) throw new AssertionError("constructor access changed");
            Object receiver = constructor.newInstance();
            Method compare = target.getDeclaredMethod("assignAndCompare", int.class);
            Field calls = target.getField("calls");
            Field z = target.getField("z");
            boolean answer = ((Boolean) compare.invoke(receiver, 2)).booleanValue();
            int observedCalls = calls.getInt(receiver);
            int observedZ = z.getInt(receiver);
            if (answer || observedCalls != 1 || observedZ != 3) {
                throw new AssertionError("unexpected effect: result=" + answer + " calls=" + observedCalls + " z=" + observedZ);
            }
            System.out.println("verified methods=3 assignAndCompare(2)=false calls=1 z=3");
        }
    }
}

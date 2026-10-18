import java.lang.reflect.Constructor;
import java.lang.reflect.Field;
import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.net.URL;
import java.net.URLClassLoader;
import java.nio.file.Paths;
import java.security.CodeSource;

/** Isolated observations for TestLoopConditionInvoke$TestCls.test(char). */
public final class TestLoopConditionInvokeDriver {
    private static final String TARGET = "jadx.tests.integration.loops.TestLoopConditionInvoke$TestCls";

    private static final class Case {
        final String name;
        final int startingPos;
        final char lastChar;
        Case(String name, int startingPos, char lastChar) {
            this.name = name;
            this.startingPos = startingPos;
            this.lastChar = lastChar;
        }
    }

    private static final Case[] CASES = {
        new Case("zero-char-pos-zero", 0, (char) 0),
        new Case("zero-char-pos-seventeen", 17, (char) 0),
        new Case("nonzero-char-pos-zero", 0, 'x'),
        new Case("nonzero-char-pos-seventeen", 17, 'x')
    };

    private static String codeSource(Class<?> target) throws Exception {
        CodeSource source = target.getProtectionDomain().getCodeSource();
        if (source == null || source.getLocation() == null) {
            throw new IllegalStateException("target has no CodeSource");
        }
        return Paths.get(source.getLocation().toURI()).toRealPath().toString();
    }

    public static void main(String[] args) throws Exception {
        if (args.length != 2) {
            throw new IllegalArgumentException("usage: TestLoopConditionInvokeDriver actualRoot expectedRoot");
        }
        String actual = Paths.get(args[0]).toRealPath().toString();
        String expected = Paths.get(args[1]).toRealPath().toString();
        URL url = Paths.get(actual).toRealPath().toUri().toURL();
        try (URLClassLoader loader = new URLClassLoader(new URL[] {url}, null)) {
            Class<?> target = Class.forName(TARGET, true, loader);
            if (target.getClassLoader() != loader) {
                throw new IllegalStateException("target did not load from parentless loader");
            }
            String actualCodeSource = codeSource(target);
            if (!actualCodeSource.equals(expected)) {
                throw new IllegalStateException("wrong CodeSource: expected=" + expected + " actual=" + actualCodeSource);
            }
            Constructor<?> constructor = target.getDeclaredConstructor();
            Object instance = constructor.newInstance();
            Field pos = target.getDeclaredField("pos");
            pos.setAccessible(true);
            Method test = target.getDeclaredMethod("test", char.class);
            for (int i = 0; i < CASES.length; i++) {
                Case c = CASES[i];
                pos.setInt(instance, c.startingPos);
                int before = pos.getInt(instance);
                Object result = null;
                Throwable failure = null;
                try {
                    result = test.invoke(instance, Character.valueOf(c.lastChar));
                } catch (InvocationTargetException e) {
                    failure = e.getCause();
                }
                int after = pos.getInt(instance);
                String exception = failure == null ? "none" : failure.getClass().getName();
                String value = failure == null ? String.valueOf(result) : "<none>";
                System.out.println("CASE index=" + i + " name=" + c.name
                        + " lastChar=" + (int) c.lastChar
                        + " result=" + value
                        + " exception=" + exception
                        + " posBefore=" + before
                        + " posAfter=" + after);
            }
        }
    }
}

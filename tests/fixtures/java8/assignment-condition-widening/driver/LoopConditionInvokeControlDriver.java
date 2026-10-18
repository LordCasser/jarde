import java.lang.reflect.Constructor;
import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.net.URL;
import java.net.URLClassLoader;
import java.nio.file.Paths;
import java.security.CodeSource;

/** Parentless target loader; emits deterministic rows for six bounded controls. */
public final class LoopConditionInvokeControlDriver {
    private static final String TARGET = "controls.LoopConditionInvokeControl";

    private static final class Case {
        final String name;
        final char[] input;
        final int startingPos;
        final char lastChar;

        Case(String name, char[] input, int startingPos, char lastChar) {
            this.name = name;
            this.input = input;
            this.startingPos = startingPos;
            this.lastChar = lastChar;
        }
    }

    private static final Case[] CASES = {
        new Case("hit-first", new char[] {'x', 0}, 0, 'x'),
        new Case("nonmatch-then-hit", new char[] {'a', 'x', 0}, 0, 'x'),
        new Case("nonzero-start-stop-restores", new char[] {'z', 'a', 0, 'x'}, 1, 'x'),
        new Case("stop-precedes-equal-target", new char[] {0}, 0, (char) 0),
        new Case("missing-stop-bounds-failure", new char[] {'a'}, 0, 'x'),
        new Case("maximum-char-hit", new char[] {(char) 65535, 0}, 0, (char) 65535)
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
            throw new IllegalArgumentException("usage: LoopConditionInvokeControlDriver actualRoot expectedRoot");
        }
        String actual = Paths.get(args[0]).toRealPath().toString();
        String expected = Paths.get(args[1]).toRealPath().toString();
        URL url = Paths.get(actual).toUri().toURL();
        try (URLClassLoader loader = new URLClassLoader(new URL[] {url}, null)) {
            Class<?> target = Class.forName(TARGET, true, loader);
            if (target.getClassLoader() != loader) {
                throw new IllegalStateException("target did not load from parentless loader");
            }
            String actualCodeSource = codeSource(target);
            if (!actualCodeSource.equals(expected)) {
                throw new IllegalStateException("wrong CodeSource: expected=" + expected + " actual=" + actualCodeSource);
            }
            Constructor<?> ctor = target.getConstructor(char[].class, int.class);
            Method test = target.getMethod("test", char.class);
            Method getPos = target.getMethod("getPos");
            Method getCalls = target.getMethod("getCalls");
            Method getEvents = target.getMethod("getEvents");
            for (int i = 0; i < CASES.length; i++) {
                Case c = CASES[i];
                Object instance = ctor.newInstance(c.input, Integer.valueOf(c.startingPos));
                Object result = null;
                Throwable failure = null;
                try {
                    result = test.invoke(instance, Character.valueOf(c.lastChar));
                } catch (InvocationTargetException e) {
                    failure = e.getCause();
                }
                int after = ((Integer) getPos.invoke(instance)).intValue();
                int calls = ((Integer) getCalls.invoke(instance)).intValue();
                int events = ((Integer) getEvents.invoke(instance)).intValue();
                String exception = failure == null ? "none" : failure.getClass().getSimpleName();
                String value = failure == null ? String.valueOf(result) : "<none>";
                System.out.println("CASE index=" + i + " name=" + c.name
                        + " lastChar=" + (int) c.lastChar
                        + " result=" + value
                        + " exception=" + exception
                        + " posBefore=" + c.startingPos
                        + " posAfter=" + after
                        + " calls=" + calls
                        + " events=" + events);
            }
        }
    }
}

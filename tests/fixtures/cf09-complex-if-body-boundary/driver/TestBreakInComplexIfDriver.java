import java.io.ByteArrayOutputStream;
import java.io.PrintStream;
import java.io.PrintWriter;
import java.io.StringWriter;
import java.lang.reflect.Constructor;
import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Field;
import java.lang.reflect.Method;
import java.net.URL;
import java.net.URLClassLoader;
import java.nio.charset.StandardCharsets;
import java.nio.file.Paths;
import java.security.CodeSource;
import java.util.Base64;
import java.util.LinkedHashMap;
import java.util.Map;
import java.util.TreeMap;

/** Parentless finite observations of TestBreakInComplexIf$TestCls.test(Map,int). */
public final class TestBreakInComplexIfDriver {
    private static final String TARGET = "jadx.tests.integration.loops.TestBreakInComplexIf$TestCls";
    private static final String[] NAMES = {
        "source-check-first-map", "source-check-complex-break", "null-point-short-circuit",
        "missing-first-key", "last-allowed-index-99", "exclusive-upper-bound-does-not-dereference-map",
        "null-map-in-range", "bounded-full-traversal", "wrapped-start-breaks-immediately"
    };
    private static final int[] MAP_X = {2, 2, 2, 2, 98, 99, 98, 0, Integer.MAX_VALUE};
    private static String codeSource(Class<?> c) throws Exception {
        CodeSource source = c.getProtectionDomain().getCodeSource();
        if (source == null || source.getLocation() == null) throw new IllegalStateException("target has no CodeSource");
        return Paths.get(source.getLocation().toURI()).toRealPath().toString();
    }
    private static String b64(String s) { return Base64.getEncoder().encodeToString(s.getBytes(StandardCharsets.UTF_8)); }
    private static void requireEmptyCwd(String expected) throws Exception {
        String actual = Paths.get(".").toRealPath().toString();
        if (!actual.equals(expected)) throw new IllegalStateException("unexpected JVM cwd: expected=" + expected + " actual=" + actual);
        String[] entries = new java.io.File(".").list();
        if (entries == null || entries.length != 0) throw new IllegalStateException("JVM cwd is not empty");
    }
    private static Object point(Constructor<?> ctor, Object receiver, int x, int y) throws Exception {
        return ctor.newInstance(receiver, Integer.valueOf(x), Integer.valueOf(y));
    }
    private static Map<String, Object> entries(int index, Object receiver, Constructor<?> pointCtor) throws Exception {
        if (index == 5 || index == 6) return null;
        Map<String, Object> map = new LinkedHashMap<String, Object>();
        switch (index) {
            case 0: map.put("3", point(pointCtor, receiver, 100, 100)); map.put("4", point(pointCtor, receiver, 60, 100)); break;
            case 1: map.put("3", point(pointCtor, receiver, 100, 100)); map.put("4", point(pointCtor, receiver, 60, 0)); map.put("5", point(pointCtor, receiver, 60, 100)); break;
            case 2: map.put("3", null); map.put("4", point(pointCtor, receiver, 1, 100)); break;
            case 3: break;
            case 4: map.put("99", point(pointCtor, receiver, 0, 100)); break;
            case 7:
                for (int x = 1; x <= 99; x++) map.put(String.valueOf(x), point(pointCtor, receiver, x, 100));
                break;
            case 8: map.put(String.valueOf(Integer.MIN_VALUE), point(pointCtor, receiver, 17, 100));
                    map.put(String.valueOf(Integer.MIN_VALUE + 1), point(pointCtor, receiver, 18, 0)); break;
            default: throw new IllegalArgumentException("unknown case index " + index);
        }
        return map;
    }
    private static int pointField(Object value, String name) throws Exception {
        Field field = value.getClass().getField(name);
        field.setAccessible(true);
        return field.getInt(value);
    }
    private static String snapshot(Map<String, Object> map) throws Exception {
        if (map == null) return "null";
        StringBuilder b = new StringBuilder("{");
        boolean first = true;
        for (Map.Entry<String, Object> e : new TreeMap<String, Object>(map).entrySet()) {
            if (!first) b.append(','); first = false;
            b.append(e.getKey()).append('=');
            Object value = e.getValue();
            if (value == null) b.append("null");
            else {
                Class<?> c = value.getClass();
                b.append("Point(").append(pointField(value, "x")).append(',')
                  .append(pointField(value, "y")).append(')');
            }
        }
        return b.append('}').toString();
    }
    private static void observe(Method method, Constructor<?> targetCtor, Constructor<?> pointCtor,
                                int index, String cwd, PrintStream resultOut) throws Exception {
        requireEmptyCwd(cwd);
        Object receiver = targetCtor.newInstance();
        Map<String, Object> map = entries(index, receiver, pointCtor);
        String inputSnapshot = "mapX=" + MAP_X[index] + ";map=" + snapshot(map);
        PrintStream previous = System.out;
        ByteArrayOutputStream captured = new ByteArrayOutputStream();
        PrintStream targetOut = new PrintStream(captured, true, "UTF-8");
        Throwable cause = null; String stack = ""; String result = "<not-returned>";
        try {
            System.setOut(targetOut);
            try { result = "return=" + String.valueOf(method.invoke(receiver, map, Integer.valueOf(MAP_X[index]))); }
            catch (InvocationTargetException e) {
                cause = e.getCause(); StringWriter w = new StringWriter();
                cause.printStackTrace(new PrintWriter(w)); stack = w.toString();
            }
        } finally { targetOut.flush(); System.setOut(previous); targetOut.close(); }
        String afterSnapshot = snapshot(map);
        requireEmptyCwd(cwd);
        String exception = cause == null ? "none" : cause.getClass().getName();
        String message = cause == null || cause.getMessage() == null ? "" : cause.getMessage();
        String output = new String(captured.toByteArray(), StandardCharsets.UTF_8);
        resultOut.println("CASE index=" + index + " name=" + NAMES[index] + " input_b64=" + b64(inputSnapshot)
            + " result_b64=" + b64(result) + " exception=" + exception + " exception_message_b64=" + b64(message)
            + " exception_stack_b64=" + b64(stack) + " target_stdout_b64=" + b64(output)
            + " after_b64=" + b64(afterSnapshot));
    }
    public static void main(String[] args) throws Exception {
        if (args.length != 3) throw new IllegalArgumentException("usage: TestBreakInComplexIfDriver actualRoot expectedRoot emptyWorkingDirectory");
        String actual = Paths.get(args[0]).toRealPath().toString();
        String expected = Paths.get(args[1]).toRealPath().toString();
        String cwd = Paths.get(args[2]).toRealPath().toString();
        requireEmptyCwd(cwd);
        try (URLClassLoader loader = new URLClassLoader(new URL[]{Paths.get(actual).toUri().toURL()}, null)) {
            Class<?> target = Class.forName(TARGET, true, loader);
            if (target.getClassLoader() != loader) throw new IllegalStateException("target did not load from parentless loader");
            String origin = codeSource(target);
            if (!origin.equals(expected)) throw new IllegalStateException("wrong CodeSource: expected=" + expected + " actual=" + origin);
            Class<?> pointClass = Class.forName(TARGET + "$Point", true, loader);
            if (pointClass.getClassLoader() != loader) throw new IllegalStateException("Point did not load from target parentless loader");
            String pointOrigin = codeSource(pointClass);
            if (!pointOrigin.equals(expected)) throw new IllegalStateException("wrong Point CodeSource: expected=" + expected + " actual=" + pointOrigin);
            Method test = target.getDeclaredMethod("test", Map.class, int.class);
            test.setAccessible(true);
            Constructor<?> targetCtor = target.getDeclaredConstructor(); targetCtor.setAccessible(true);
            Constructor<?> pointCtor = pointClass.getDeclaredConstructor(target, int.class, int.class); pointCtor.setAccessible(true);
            PrintStream resultOut = System.out;
            for (int i = 0; i < NAMES.length; i++) observe(test, targetCtor, pointCtor, i, cwd, resultOut);
            requireEmptyCwd(cwd);
        }
    }
}

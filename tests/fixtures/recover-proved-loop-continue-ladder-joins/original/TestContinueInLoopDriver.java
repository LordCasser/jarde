import java.io.ByteArrayOutputStream;
import java.io.PrintStream;
import java.io.PrintWriter;
import java.io.StringWriter;
import java.lang.reflect.Constructor;
import java.lang.reflect.Field;
import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.lang.reflect.Modifier;
import java.net.URL;
import java.net.URLClassLoader;
import java.nio.charset.StandardCharsets;
import java.nio.file.Paths;
import java.security.CodeSource;
import java.util.Base64;

/** Parentless bounded observations of the untouched TestContinueInLoop$TestCls.test method. */
public final class TestContinueInLoopDriver {
    private static final String TARGET = "jadx.tests.integration.loops.TestContinueInLoop$TestCls";
    private static final String[] NAMES = {
        "increment-then-break", "decrement-then-break", "continue-update-then-later-break",
        "two-continues-run-two-updates", "negative-bound-full-traversal-and-nonzero-f",
        "empty-array-still-increments-f", "null-array-throws-before-f-update", "f-increment-wraps-as-java-int"
    };
    private static final int[][] ARRAYS = {
        {-1, 9}, {2, 7}, {2, 0, 4}, {1, 1, 0}, {-2, 0}, {}, null, {}
    };
    private static final int[] BOUNDS = {2, 1, 2, 1, -1, 0, 0, -5};
    private static final int[] INITIAL_F = {3, -4, 10, 0, 41, 7, 9, Integer.MAX_VALUE};

    private static String codeSource(Class<?> c) throws Exception {
        CodeSource s = c.getProtectionDomain().getCodeSource();
        if (s == null || s.getLocation() == null) throw new IllegalStateException("target has no CodeSource");
        return Paths.get(s.getLocation().toURI()).toRealPath().toString();
    }
    private static String b64(String s) {
        return Base64.getEncoder().encodeToString(s.getBytes(StandardCharsets.UTF_8));
    }
    private static String array(int[] a) {
        if (a == null) return "<null>";
        StringBuilder b = new StringBuilder("[");
        for (int i = 0; i < a.length; i++) { if (i > 0) b.append(','); b.append(a[i]); }
        return b.append(']').toString();
    }
    private static String describeInput(int i) {
        return "array=" + array(ARRAYS[i]) + ";b=" + BOUNDS[i] + ";f=" + INITIAL_F[i];
    }
    private static void requireEmptyCwd(String expected) throws Exception {
        String actual = Paths.get(".").toRealPath().toString();
        if (!actual.equals(expected)) throw new IllegalStateException("unexpected JVM cwd: expected=" + expected + " actual=" + actual);
        String[] entries = new java.io.File(".").list();
        if (entries == null || entries.length != 0) throw new IllegalStateException("JVM cwd is not empty");
    }
    private static void observe(Method method, Constructor<?> constructor, Field field, int i,
                                String cwd, PrintStream resultOut) throws Exception {
        requireEmptyCwd(cwd);
        Object receiver = constructor.newInstance();
        field.setInt(receiver, INITIAL_F[i]);
        String initialInput = describeInput(i);
        PrintStream previous = System.out;
        ByteArrayOutputStream captured = new ByteArrayOutputStream();
        PrintStream targetOut = new PrintStream(captured, true, "UTF-8");
        Throwable cause = null; String stack = "";
        try {
            System.setOut(targetOut);
            try { method.invoke(receiver, (Object) ARRAYS[i], BOUNDS[i]); }
            catch (InvocationTargetException e) {
                cause = e.getCause(); StringWriter w = new StringWriter();
                cause.printStackTrace(new PrintWriter(w)); stack = w.toString();
            }
        } finally { targetOut.flush(); System.setOut(previous); targetOut.close(); }
        requireEmptyCwd(cwd);
        String exception = cause == null ? "none" : cause.getClass().getName();
        String message = cause == null || cause.getMessage() == null ? "" : cause.getMessage();
        String result = "array=" + array(ARRAYS[i]) + ";f=" + field.getInt(receiver);
        String output = new String(captured.toByteArray(), StandardCharsets.UTF_8);
        resultOut.println("CASE index=" + i + " name=" + NAMES[i] + " input_b64=" + b64(initialInput)
            + " result_b64=" + b64(result) + " exception=" + exception + " exception_message_b64=" + b64(message)
            + " exception_stack_b64=" + b64(stack) + " target_stdout_b64=" + b64(output));
    }
    public static void main(String[] args) throws Exception {
        if (args.length != 3) throw new IllegalArgumentException("usage: TestContinueInLoopDriver actualRoot expectedRoot emptyWorkingDirectory");
        String actual = Paths.get(args[0]).toRealPath().toString();
        String expected = Paths.get(args[1]).toRealPath().toString();
        String cwd = Paths.get(args[2]).toRealPath().toString();
        requireEmptyCwd(cwd);
        try (URLClassLoader loader = new URLClassLoader(new URL[]{Paths.get(actual).toUri().toURL()}, null)) {
            Class<?> target = Class.forName(TARGET, true, loader);
            if (target.getClassLoader() != loader) throw new IllegalStateException("target did not load from parentless loader");
            String origin = codeSource(target);
            if (!origin.equals(expected)) throw new IllegalStateException("wrong CodeSource: expected=" + expected + " actual=" + origin);
            java.util.Set<String> inventory = new java.util.TreeSet<String>();
            for (Method m : target.getDeclaredMethods()) inventory.add(m.getName() + TypeDescriptor.of(m));
            for (Constructor<?> c : target.getDeclaredConstructors()) inventory.add("<init>" + TypeDescriptor.of(c));
            if (!inventory.equals(expectedMethodsWithCtor())) throw new IllegalStateException("target declared method inventory differs: " + inventory);
            if (!Modifier.isPublic(target.getModifiers())) throw new IllegalStateException("target class is not public");
            Constructor<?> constructor = target.getDeclaredConstructor();
            if (!Modifier.isPublic(constructor.getModifiers())) throw new IllegalStateException("target constructor is not public");
            Field f = target.getDeclaredField("f");
            if (f.getType() != int.class || !Modifier.isPublic(f.getModifiers()) || Modifier.isStatic(f.getModifiers()))
                throw new IllegalStateException("f is not the expected public instance int field");
            if (target.getDeclaredFields().length != 1) throw new IllegalStateException("target field inventory differs");
            Method test = target.getDeclaredMethod("test", int[].class, int.class);
            if (!Modifier.isPublic(test.getModifiers()) || Modifier.isStatic(test.getModifiers()))
                throw new IllegalStateException("test(int[],int) does not have the expected public instance access");
            PrintStream resultOut = System.out;
            for (int i = 0; i < NAMES.length; i++) observe(test, constructor, f, i, cwd, resultOut);
            requireEmptyCwd(cwd);
        }
    }
    private static java.util.Set<String> expectedMethodsWithCtor() {
        java.util.Set<String> expected = new java.util.TreeSet<String>();
        expected.add("<init>()V"); expected.add("test([II)V"); return expected;
    }
    private static final class TypeDescriptor {
        static String of(java.lang.reflect.Executable e) {
            StringBuilder b = new StringBuilder("(");
            for (Class<?> p : e.getParameterTypes()) b.append(type(p));
            b.append(')');
            return b.append(e instanceof Method ? type(((Method) e).getReturnType()) : "V").toString();
        }
        static String type(Class<?> c) {
            if (c.isArray()) return c.getName().replace('.', '/');
            if (!c.isPrimitive()) return "L" + c.getName().replace('.', '/') + ";";
            if (c == void.class) return "V"; if (c == boolean.class) return "Z"; if (c == byte.class) return "B";
            if (c == char.class) return "C"; if (c == short.class) return "S"; if (c == int.class) return "I";
            if (c == long.class) return "J"; if (c == float.class) return "F"; if (c == double.class) return "D";
            throw new IllegalArgumentException("unsupported primitive: " + c);
        }
    }
}

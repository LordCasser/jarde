import java.io.ByteArrayOutputStream;
import java.io.PrintStream;
import java.io.PrintWriter;
import java.io.StringWriter;
import java.lang.reflect.Constructor;
import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.lang.reflect.Modifier;
import java.net.URL;
import java.net.URLClassLoader;
import java.nio.charset.StandardCharsets;
import java.nio.file.Paths;
import java.security.CodeSource;
import java.util.Base64;

/** Parentless finite observations of the untouched TestDoWhileBreak$TestCls.test method. */
public final class TestDoWhileBreakDriver {
    private static final String TARGET = "jadx.tests.integration.loops.TestDoWhileBreak$TestCls";
    private static final String[] NAMES = {
        "strict-threshold-do-body-latch", "break-sets-return-local-before-exit"
    };
    private static final int[] K = {9, 10};

    private static String codeSource(Class<?> c) throws Exception {
        CodeSource s = c.getProtectionDomain().getCodeSource();
        if (s == null || s.getLocation() == null) throw new IllegalStateException("target has no CodeSource");
        return Paths.get(s.getLocation().toURI()).toRealPath().toString();
    }
    private static String b64(String s) {
        return Base64.getEncoder().encodeToString(s.getBytes(StandardCharsets.UTF_8));
    }
    private static void requireEmptyCwd(String expected) throws Exception {
        String actual = Paths.get(".").toRealPath().toString();
        if (!actual.equals(expected)) throw new IllegalStateException("unexpected JVM cwd: expected=" + expected + " actual=" + actual);
        String[] entries = new java.io.File(".").list();
        if (entries == null || entries.length != 0) throw new IllegalStateException("JVM cwd is not empty");
    }
    private static void observe(Method method, Constructor<?> constructor, int index, String cwd, PrintStream resultOut) throws Exception {
        requireEmptyCwd(cwd);
        Object receiver = constructor.newInstance();
        String inputSnapshot = "k=" + K[index];
        PrintStream previous = System.out;
        ByteArrayOutputStream captured = new ByteArrayOutputStream();
        PrintStream targetOut = new PrintStream(captured, true, "UTF-8");
        Throwable cause = null; String stack = ""; String result = "<not-returned>";
        try {
            System.setOut(targetOut);
            try { result = "return=" + String.valueOf(method.invoke(receiver, K[index])); }
            catch (InvocationTargetException e) {
                cause = e.getCause(); StringWriter w = new StringWriter();
                cause.printStackTrace(new PrintWriter(w)); stack = w.toString();
            }
        } finally { targetOut.flush(); System.setOut(previous); targetOut.close(); }
        requireEmptyCwd(cwd);
        String exception = cause == null ? "none" : cause.getClass().getName();
        String message = cause == null || cause.getMessage() == null ? "" : cause.getMessage();
        String output = new String(captured.toByteArray(), StandardCharsets.UTF_8);
        resultOut.println("CASE index=" + index + " name=" + NAMES[index] + " input_b64=" + b64(inputSnapshot)
            + " result_b64=" + b64(result) + " exception=" + exception + " exception_message_b64=" + b64(message)
            + " exception_stack_b64=" + b64(stack) + " target_stdout_b64=" + b64(output));
    }
    public static void main(String[] args) throws Exception {
        if (args.length != 3) throw new IllegalArgumentException("usage: TestDoWhileBreakDriver actualRoot expectedRoot emptyWorkingDirectory");
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
            if (target.getDeclaredFields().length != 0) throw new IllegalStateException("target field inventory differs");
            Method test = target.getDeclaredMethod("test", int.class);
            if (!Modifier.isPublic(test.getModifiers()) || Modifier.isStatic(test.getModifiers()) || test.getReturnType() != int.class)
                throw new IllegalStateException("test(int) does not have the expected public instance int access");
            PrintStream resultOut = System.out;
            for (int i = 0; i < NAMES.length; i++) observe(test, constructor, i, cwd, resultOut);
            requireEmptyCwd(cwd);
        }
    }
    private static java.util.Set<String> expectedMethodsWithCtor() {
        java.util.Set<String> expected = new java.util.TreeSet<String>();
        expected.add("<init>()V"); expected.add("test(I)I"); return expected;
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

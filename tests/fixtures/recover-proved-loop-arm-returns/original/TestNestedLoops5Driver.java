import java.io.File;
import java.lang.reflect.Constructor;
import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.net.URI;
import java.security.CodeSource;

public final class TestNestedLoops5Driver {
    private static final String TARGET = "jadx.tests.integration.loops.TestNestedLoops5$TestCls";

    private TestNestedLoops5Driver() {
    }

    public static void main(String[] args) throws Exception {
        if (args.length != 2) {
            throw new IllegalArgumentException("usage: TestNestedLoops5Driver <expected-class-root> <core|check>");
        }
        Class<?> target = Class.forName(TARGET);
        CodeSource source = target.getProtectionDomain().getCodeSource();
        if (source == null || source.getLocation() == null) {
            throw new AssertionError("target has no CodeSource: " + TARGET);
        }
        File actualRoot = new File(new URI(source.getLocation().toString())).getCanonicalFile();
        File expectedRoot = new File(args[0]).getCanonicalFile();
        if (!actualRoot.equals(expectedRoot)) {
            throw new AssertionError("target loaded from " + actualRoot + ", expected " + expectedRoot);
        }

        target.getDeclaredConstructor();
        target.getDeclaredMethod("testFor");
        target.getDeclaredMethod("check");
        if (target.getDeclaredMethods().length != 2) {
            throw new AssertionError("expected exactly testFor() and check() methods");
        }

        Constructor<?> constructor = target.getDeclaredConstructor();
        Method testFor = target.getDeclaredMethod("testFor");
        Method check = target.getDeclaredMethod("check");
        Object same = constructor.newInstance();
        if ("check".equals(args[1])) {
            invoke(check, same);
            System.out.println("CASE check result=passed");
        } else if ("core".equals(args[1])) {
            row("fresh-testFor", invoke(testFor, same));
            row("same-instance-second-testFor", invoke(testFor, same));
            row("same-instance-third-testFor", invoke(testFor, same));
        } else {
            throw new IllegalArgumentException("unknown driver mode: " + args[1]);
        }
    }

    private static Object invoke(Method method, Object receiver) throws Exception {
        try {
            return method.invoke(receiver);
        } catch (InvocationTargetException e) {
            Throwable cause = e.getCause();
            if (cause instanceof Exception) {
                throw (Exception) cause;
            }
            if (cause instanceof Error) {
                throw (Error) cause;
            }
            throw new AssertionError("unexpected invocation throwable", cause);
        }
    }

    private static void row(String label, Object result) {
        System.out.println("CASE " + label + " result=" + result);
    }
}

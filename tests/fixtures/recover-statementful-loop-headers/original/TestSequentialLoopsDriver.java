import java.io.File;
import java.lang.reflect.Constructor;
import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.net.URI;
import java.security.CodeSource;

public final class TestSequentialLoopsDriver {
    private static final String TARGET = "jadx.tests.integration.loops.TestSequentialLoops$TestCls";

    private TestSequentialLoopsDriver() {
    }

    public static void main(String[] args) throws Exception {
        if (args.length != 1) {
            throw new IllegalArgumentException("usage: TestSequentialLoopsDriver <expected-class-root>");
        }
        Class<?> target = Class.forName(TARGET);
        CodeSource codeSource = target.getProtectionDomain().getCodeSource();
        if (codeSource == null || codeSource.getLocation() == null) {
            throw new AssertionError("target has no CodeSource: " + TARGET);
        }
        File actualRoot = new File(new URI(codeSource.getLocation().toString())).getCanonicalFile();
        File expectedRoot = new File(args[0]).getCanonicalFile();
        if (!actualRoot.equals(expectedRoot)) {
            throw new AssertionError("target loaded from " + actualRoot + ", expected " + expectedRoot);
        }

        Constructor<?> constructor = target.getDeclaredConstructor();
        Method test = target.getDeclaredMethod("test", int.class, int.class);
        if (target.getDeclaredMethods().length != 1 || test.getReturnType() != int.class) {
            throw new AssertionError("expected exactly test(II)I on " + TARGET);
        }
        Object sameInstance = constructor.newInstance();
        int[][] inputs = {
                {-1, 10},
                {-2, 20},
                {-3, 20},
                {Integer.MAX_VALUE, 0},
                {Integer.MIN_VALUE + 8, 0},
                {Integer.MAX_VALUE, -10},
        };
        for (int index = 0; index < inputs.length; index++) {
            int a = inputs[index][0];
            int b = inputs[index][1];
            Object result = invoke(test, sameInstance, a, b);
            if (!(result instanceof Integer)) {
                throw new AssertionError("test(II)I returned a non-int result at case " + index);
            }
            System.out.println("CASE " + index + " input=" + a + "," + b + " result=" + result);
        }
    }

    private static Object invoke(Method method, Object receiver, int a, int b) throws Exception {
        try {
            return method.invoke(receiver, a, b);
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
}

import java.io.File;
import java.lang.reflect.Constructor;
import java.lang.reflect.Field;
import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.net.URI;
import java.security.CodeSource;

public final class ParameterizedNestedReturnDriver {
    private static final String[] STATE_FIELDS = {
            "innerVisits", "tailVisits", "tailSum", "earlyReturns", "rolling"
    };

    private ParameterizedNestedReturnDriver() {
    }

    public static void main(String[] args) throws Exception {
        if (args.length != 2) {
            throw new IllegalArgumentException("usage: ParameterizedNestedReturnDriver <target-class> <expected-class-root>");
        }
        Class<?> target = Class.forName(args[0]);
        verifyCodeSource(target, args[1]);
        Constructor<?> constructor = target.getDeclaredConstructor();
        Method run = target.getDeclaredMethod("run", int.class, int.class, int.class, int.class, int.class);
        Object same = constructor.newInstance();
        int[][] cases = casesFor(args[0]);
        for (int index = 0; index < cases.length; index++) {
            Object result = invoke(run, same, cases[index]);
            System.out.println("CASE " + args[0] + "-" + (index + 1)
                    + " result=" + result + " state=" + snapshot(target, same));
        }
    }

    private static int[][] casesFor(String name) {
        if (name.equals("NestedReturnA$TestCls")) {
            return new int[][] {
                    { 4, 2, 3, 4, 2 },
                    { 4, 2, 3, -1, -1 },
                    { 4, 2, 3, 3, 3 }
            };
        }
        if (name.equals("NestedReturnB$TestCls")) {
            return new int[][] {
                    { 4, 3, 2, 1, 4 },
                    { -1, 3, 2, -1, 4 },
                    { 3, 3, 2, 2, 4 }
            };
        }
        throw new IllegalArgumentException("unsupported target class: " + name);
    }

    private static void verifyCodeSource(Class<?> target, String expectedRoot) throws Exception {
        CodeSource source = target.getProtectionDomain().getCodeSource();
        if (source == null || source.getLocation() == null) {
            throw new AssertionError("target has no CodeSource: " + target.getName());
        }
        File actual = new File(new URI(source.getLocation().toString())).getCanonicalFile();
        File expected = new File(expectedRoot).getCanonicalFile();
        if (!actual.equals(expected)) {
            throw new AssertionError("target loaded from " + actual + ", expected " + expected);
        }
    }

    private static Object invoke(Method method, Object receiver, int[] arguments) throws Exception {
        try {
            return method.invoke(receiver, arguments[0], arguments[1], arguments[2], arguments[3], arguments[4]);
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

    private static String snapshot(Class<?> target, Object receiver) throws Exception {
        StringBuilder result = new StringBuilder();
        for (String name : STATE_FIELDS) {
            Field field = target.getDeclaredField(name);
            field.setAccessible(true);
            if (result.length() != 0) {
                result.append(',');
            }
            result.append(name).append('=').append(field.getInt(receiver));
        }
        return result.toString();
    }
}

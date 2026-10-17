import java.io.File;
import java.lang.reflect.Constructor;
import java.lang.reflect.Field;
import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.net.URI;
import java.security.CodeSource;

public final class RetryFixtureDriver {
    private RetryFixtureDriver() {
    }

    public static void main(String[] args) throws Exception {
        if (args.length != 2) {
            throw new IllegalArgumentException("usage: RetryFixtureDriver <binary-class-name> <expected-class-root>");
        }
        Class<?> target = Class.forName(args[0]);
        CodeSource codeSource = target.getProtectionDomain().getCodeSource();
        if (codeSource == null || codeSource.getLocation() == null) {
            throw new AssertionError("target has no CodeSource: " + args[0]);
        }
        File actualRoot = new File(new URI(codeSource.getLocation().toString())).getCanonicalFile();
        File expectedRoot = new File(args[1]).getCanonicalFile();
        if (!actualRoot.equals(expectedRoot)) {
            throw new AssertionError("target loaded from " + actualRoot + ", expected " + expectedRoot);
        }

        Constructor<?> constructor = target.getConstructor();
        Method execute = target.getMethod("execute", Integer.TYPE, Integer.TYPE, Integer.TYPE);
        runCase(target, constructor, execute, "direct-success", 0, 1, 0);
        runCase(target, constructor, execute, "caught-then-inner-gateway-exit-limit1", 1, 1, 0);
        runCase(target, constructor, execute, "caught-then-inner-gateway-exit-limit2", 1, 2, 0);
        runCase(target, constructor, execute, "normal-success-then-second-outer-prefix", 0, 2, 0);
        runCase(target, constructor, execute, "unmatched-error-escapes", 0, 1, 4);
        runCase(target, constructor, execute, "f1-throws-outside-row", 0, 1, 1);
        runCase(target, constructor, execute, "f2-throws-outside-row", 0, 1, 2);
        runCase(target, constructor, execute, "catch-effect-throws", 1, 1, 8);
        runRepeated(target, constructor, execute);
    }

    private static void runCase(Class<?> target, Constructor<?> constructor, Method execute,
            String name, int failures, int outerLimit, int mode) throws Exception {
        Object receiver = constructor.newInstance();
        Observation observation = invoke(execute, receiver, failures, outerLimit, mode);
        print(target, name, failures, outerLimit, mode, receiver, observation);
    }

    private static void runRepeated(Class<?> target, Constructor<?> constructor, Method execute) throws Exception {
        Object receiver = constructor.newInstance();
        Observation first = invoke(execute, receiver, 0, 1, 0);
        print(target, "same-instance-first", 0, 1, 0, receiver, first);
        Observation second = invoke(execute, receiver, 0, 1, 0);
        print(target, "same-instance-second", 0, 1, 0, receiver, second);
    }

    private static Observation invoke(Method execute, Object receiver,
            int failures, int outerLimit, int mode) throws Exception {
        try {
            Object value = execute.invoke(receiver, Integer.valueOf(failures), Integer.valueOf(outerLimit), Integer.valueOf(mode));
            return new Observation(value == null ? "null" : value.toString(), null);
        } catch (InvocationTargetException e) {
            return new Observation("none", e.getCause());
        }
    }

    private static void print(Class<?> target, String name,
            int failures, int outerLimit, int mode,
            Object receiver, Observation observation) throws Exception {
        int trace = intField(target, "trace").getInt(receiver);
        int outer = intField(target, "outerPrefixCalls").getInt(receiver);
        int retry = intField(target, "retryPrefixCalls").getInt(receiver);
        int protectedCalls = intField(target, "protectedCalls").getInt(receiver);
        int catches = intField(target, "catchCalls").getInt(receiver);
        int outerIterations = intField(target, "outerIterations").getInt(receiver);
        Object lastCaught = objectField(target, "lastCaught").get(receiver);
        Object message = objectField(target, "lastCaughtMessage").get(receiver);
        String caughtType = lastCaught == null ? "none" : ((Throwable) lastCaught).getClass().getName();
        String caughtMessage = message == null ? "none" : clean(message.toString());
        Throwable thrown = observation.thrown;
        String thrownType = thrown == null ? "none" : thrown.getClass().getName();
        Object expectedIdentity = null;
        if (mode == 1 || mode == 2) {
            expectedIdentity = objectField(target, "prefixEscape").get(receiver);
        } else if (mode == 4) {
            expectedIdentity = objectField(target, "protectedEscape").get(receiver);
        } else if (mode == 8) {
            expectedIdentity = objectField(target, "catchEscape").get(receiver);
        }
        boolean sameEscapeObject = expectedIdentity != null && thrown == expectedIdentity;
        System.out.println("CASE class=" + target.getName()
                + " name=" + name
                + " input=" + failures + "," + outerLimit + "," + mode
                + " result=" + observation.result
                + " thrown=" + thrownType
                + " escapeIdentity=" + sameEscapeObject
                + " trace=" + trace
                + " outerPrefixCalls=" + outer
                + " retryPrefixCalls=" + retry
                + " protectedCalls=" + protectedCalls
                + " catchCalls=" + catches
                + " outerIterations=" + outerIterations
                + " caughtType=" + caughtType
                + " caughtMessage=" + caughtMessage);
    }

    private static Field intField(Class<?> target, String name) throws Exception {
        Field field = target.getField(name);
        if (field.getType() != Integer.TYPE) {
            throw new AssertionError("expected int field " + name);
        }
        return field;
    }

    private static Field objectField(Class<?> target, String name) throws Exception {
        return target.getField(name);
    }

    private static String clean(String value) {
        return value.replace(' ', '_').replace('\n', '_').replace('\r', '_');
    }

    private static final class Observation {
        final String result;
        final Throwable thrown;

        Observation(String result, Throwable thrown) {
            this.result = result;
            this.thrown = thrown;
        }
    }
}

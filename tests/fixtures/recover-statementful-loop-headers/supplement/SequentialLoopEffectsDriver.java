import java.lang.reflect.Constructor;
import java.lang.reflect.Field;
import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.net.URL;
import java.nio.file.Path;
import java.nio.file.Paths;

public final class SequentialLoopEffectsDriver {
    private static final int[][] ORIGINAL_INPUTS = {
            {-1, 10},
            {-2, 20},
            {-3, 20},
            {Integer.MAX_VALUE, 0},
            {Integer.MIN_VALUE + 8, 0},
            {Integer.MAX_VALUE, -10}
    };
    private static final int[] ORIGINAL_RESULTS = {
            7, 8, 8, Integer.MAX_VALUE, 16, 2147483637
    };

    private static Object call(Method run, Object instance, int a, int c) throws Exception {
        try {
            return run.invoke(instance, Integer.valueOf(a), Integer.valueOf(c));
        } catch (InvocationTargetException e) {
            Throwable cause = e.getCause();
            if (cause instanceof Exception) {
                throw (Exception) cause;
            }
            if (cause instanceof Error) {
                throw (Error) cause;
            }
            throw new AssertionError(cause);
        }
    }

    private static int getInt(Field f, Object obj) throws Exception {
        return f.getInt(obj);
    }

    private static void setInt(Field f, Object obj, int value) throws Exception {
        f.setInt(obj, value);
    }

    private static void require(boolean condition, String message) {
        if (!condition) {
            throw new AssertionError(message);
        }
    }

    private static String phaseDigits(long trace, int count) {
        char[] digits = new char[count];
        for (int i = count - 1; i >= 0; i--) {
            int phase = (int) (trace & 3L);
            digits[i] = (char) ('0' + phase);
            trace >>>= 2;
        }
        require(trace == 0L, "phase trace has bits beyond evaluation count");
        return new String(digits);
    }

    private static int intField(Class<?> type, String name, Object obj) throws Exception {
        return getInt(type.getField(name), obj);
    }

    public static void main(String[] args) throws Exception {
        if (args.length != 1) {
            throw new IllegalArgumentException("expected target class root");
        }
        Path expectedRoot = Paths.get(args[0]).toRealPath();
        Class<?> type = Class.forName("SequentialLoopEffects");
        URL actualLocation = type.getProtectionDomain().getCodeSource().getLocation().toURI().toURL();
        URL expectedLocation = expectedRoot.toUri().toURL();
        require(actualLocation.toURI().normalize().equals(expectedLocation.toURI().normalize()),
                "target CodeSource mismatch: " + actualLocation + " != " + expectedLocation);

        Constructor<?> constructor = type.getConstructor();
        Method run = type.getMethod("run", Integer.TYPE, Integer.TYPE);
        Field evaluations = type.getField("evaluations");
        Field phaseOneCalls = type.getField("phaseOneCalls");
        Field phaseTwoCalls = type.getField("phaseTwoCalls");
        Field lastInputC = type.getField("lastInputC");
        Field lastInputA = type.getField("lastInputA");
        Field lastValue = type.getField("lastValue");
        Field cField = type.getField("c");
        Field zField = type.getField("z");
        Field throwAt = type.getField("throwAt");
        Field overrideAt = type.getField("overrideAt");
        Field overrideValue = type.getField("overrideValue");
        Field phaseTrace = type.getField("phaseTrace");
        Field markerField = type.getField("marker");

        for (int i = 0; i < ORIGINAL_INPUTS.length; i++) {
            Object instance = constructor.newInstance();
            int a = ORIGINAL_INPUTS[i][0];
            int initialC = ORIGINAL_INPUTS[i][1];
            int result = ((Integer) call(run, instance, a, initialC)).intValue();
            require(result == ORIGINAL_RESULTS[i], "original-input result mismatch at case " + i);
            System.out.println("ORIGINAL case=" + i + " input=" + a + "," + initialC + " result=" + result);
        }

        Object shared = constructor.newInstance();
        Object marker = markerField.get(shared);
        setInt(overrideAt, shared, 3);
        setInt(overrideValue, shared, 6);
        int backZeroResult = ((Integer) call(run, shared, Integer.MAX_VALUE, Integer.MIN_VALUE + 1)).intValue();
        require(backZeroResult == 0, "back/zero-second result");
        require(intField(type, "evaluations", shared) == 3, "back/zero-second evaluation count");
        require(intField(type, "phaseOneCalls", shared) == 2, "first-loop back count");
        require(intField(type, "phaseTwoCalls", shared) == 1, "zero-body second-loop check count");
        require(intField(type, "c", shared) == 0 && intField(type, "z", shared) == 6,
                "back/zero-second fields");
        require(phaseDigits(phaseTrace.getLong(shared), 3).equals("112"), "back/zero-second phase order");
        System.out.println("EFFECT case=back-zero-second result=0 evals=3 phase1=2 phase2=1 trace=112 c=0 z=6");

        setInt(overrideAt, shared, 0);
        int multiResult = ((Integer) call(run, shared, -1, 10)).intValue();
        require(multiResult == 7, "multiple second-loop result");
        require(intField(type, "evaluations", shared) == 8, "multiple second-loop cumulative evaluations");
        require(intField(type, "phaseOneCalls", shared) == 3 && intField(type, "phaseTwoCalls", shared) == 5,
                "multiple second-loop cumulative phases");
        require(phaseDigits(phaseTrace.getLong(shared), 8).equals("11212222"), "multiple second-loop phase order");
        System.out.println("EFFECT case=multiple-second result=7 evals=8 phase1=3 phase2=5 trace=11212222");

        setInt(throwAt, shared, intField(type, "evaluations", shared) + 1);
        try {
            call(run, shared, 2, 1);
            throw new AssertionError("early throw did not escape");
        } catch (RuntimeException e) {
            require(e == marker, "early throw identity");
        }
        require(intField(type, "evaluations", shared) == 9, "early throw evaluation count");
        require(intField(type, "phaseOneCalls", shared) == 4 && intField(type, "phaseTwoCalls", shared) == 5,
                "early throw phase counts");
        require(intField(type, "c", shared) == 1 && intField(type, "lastValue", shared) == 3,
                "early throw state");
        require(phaseDigits(phaseTrace.getLong(shared), 9).equals("112122221"), "early throw phase order");
        System.out.println("EFFECT case=early-throw sameMarker=true evals=9 phase1=4 phase2=5 c=1 value=3 trace=112122221");

        setInt(throwAt, shared, intField(type, "evaluations", shared) + 5);
        try {
            call(run, shared, -1, 10);
            throw new AssertionError("late throw did not escape");
        } catch (RuntimeException e) {
            require(e == marker, "late throw identity");
        }
        require(intField(type, "evaluations", shared) == 14, "late throw evaluation count");
        require(intField(type, "phaseOneCalls", shared) == 5 && intField(type, "phaseTwoCalls", shared) == 9,
                "late throw phase counts");
        require(intField(type, "c", shared) == 7 && intField(type, "z", shared) == 7,
                "late throw leaves previous assignment intact");
        require(intField(type, "lastInputC", shared) == 7 && intField(type, "lastInputA", shared) == -1
                        && intField(type, "lastValue", shared) == 6,
                "late throw evaluated final second-loop condition");
        require(phaseDigits(phaseTrace.getLong(shared), 14).equals("11212222112222"),
                "late throw phase order");
        System.out.println("EFFECT case=late-throw sameMarker=true evals=14 phase1=5 phase2=9 c=7 z=7 value=6 trace=11212222112222");

        setInt(throwAt, shared, 0);
        int recoveryResult = ((Integer) call(run, shared, -1, 8)).intValue();
        require(recoveryResult == 7, "same-instance recovery result");
        require(intField(type, "evaluations", shared) == 17, "same-instance cumulative evaluations");
        require(intField(type, "phaseOneCalls", shared) == 6 && intField(type, "phaseTwoCalls", shared) == 11,
                "same-instance cumulative phases");
        require(phaseDigits(phaseTrace.getLong(shared), 17).equals("11212222112222122"),
                "same-instance cumulative phase order");
        System.out.println("EFFECT case=same-instance-recovery result=7 evals=17 phase1=6 phase2=11 trace=11212222112222122");
    }
}

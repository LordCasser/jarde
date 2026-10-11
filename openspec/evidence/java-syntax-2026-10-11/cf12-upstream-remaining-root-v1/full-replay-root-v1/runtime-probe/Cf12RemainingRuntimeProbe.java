import java.lang.reflect.Constructor;
import java.lang.reflect.Field;
import java.lang.reflect.Method;
import java.util.Locale;

/**
 * Small reflection-only runtime probe for the five remaining CF-12 fixtures.
 * Usage: Cf12RemainingRuntimeProbe <captured-target-class> <fixture-simple-name>
 */
public final class Cf12RemainingRuntimeProbe {
    private static final String[] SWITCH2_BOOLEAN_FIELDS = {
            "isLongtouchable",
            "isMultiTouchZoom",
            "isCanZoomIn",
            "isCanZoomOut",
            "isScrolling"
    };

    // A short balanced set: signed zero, ordinary/threshold values, infinities, and NaN.
    private static final float[] SWITCH2_FLOAT_SEEDS = {
            0.0f,
            -0.0f,
            9.5f,
            10.0f,
            10.5f,
            -10.5f,
            Float.POSITIVE_INFINITY,
            Float.NEGATIVE_INFINITY,
            Float.NaN
    };

    // Includes direct switch keys, the action&255 alias for case 5, negative aliases,
    // and default. These are intentional boundary/equivalence probes, not random values.
    private static final int[] SWITCH2_ACTIONS = {
            0, 1, 2, 5, 6, 261, -1, -256, -255, -254, -251, -250
    };

    private static final int[] SWITCH2_SEQUENCE = {
            261, 2, 6, 1, 5, 0, -251, -256
    };

    private Cf12RemainingRuntimeProbe() {
    }

    public static void main(String[] args) throws Exception {
        if (args.length != 2) {
            throw new IllegalArgumentException("expected target class and fixture simple name");
        }
        Class<?> target = Class.forName(args[0]);
        switch (args[1]) {
            case "TestSwitch2":
                testSwitch2(target);
                break;
            case "TestSwitch3":
                testSwitch3(target);
                break;
            case "TestSwitch4":
                testSwitch4(target);
                break;
            case "TestSwitchSimple":
                testSwitchSimple(target);
                break;
            case "TestSwitchWithFallThroughCase2":
                testSwitchWithFallThroughCase2(target);
                break;
            default:
                throw new IllegalArgumentException("unsupported fixture: " + args[1]);
        }
    }

    private static Object newInstance(Class<?> type) throws Exception {
        Constructor<?> constructor = type.getDeclaredConstructor();
        constructor.setAccessible(true);
        return constructor.newInstance();
    }

    private static Method declaredMethod(Class<?> type, String name, Class<?>... parameterTypes)
            throws Exception {
        Method method = type.getDeclaredMethod(name, parameterTypes);
        method.setAccessible(true);
        return method;
    }

    private static Field declaredField(Class<?> type, String name) throws Exception {
        Field field = type.getDeclaredField(name);
        field.setAccessible(true);
        return field;
    }

    private static void runCheck(Class<?> type) throws Exception {
        Method check = type.getMethod("check");
        Object target = newInstance(type);
        check.invoke(target);
        System.out.println("check=passed");
    }

    private static void testSwitch3(Class<?> type) throws Exception {
        runCheck(type);
        Method test = declaredMethod(type, "test", int.class);
        Field state = declaredField(type, "i");
        Object target = newInstance(type);
        int[] inputs = {
                Integer.MIN_VALUE, -1, 0, 1, 2, 3, 4, 10, Integer.MAX_VALUE
        };
        for (int input : inputs) {
            test.invoke(target, input);
            System.out.println("switch3 input=" + input + " i=" + state.getInt(target));
        }
    }

    private static void testSwitch4(Class<?> type) throws Exception {
        runCheck(type);
        Method parse = declaredMethod(type, "parse", char[].class, int.class, int.class);
        // Every range is an in-bounds, all-digit slice used by the original arithmetic parser.
        char[][] buffers = {
                "123".toCharArray(),
                "a=1234".toCharArray(),
                "0000".toCharArray(),
                "42".toCharArray(),
                "x7".toCharArray()
        };
        int[][] ranges = {
                {0, 3},
                {2, 4},
                {0, 4},
                {0, 2},
                {1, 1}
        };
        for (int i = 0; i < buffers.length; i++) {
            int offset = ranges[i][0];
            int length = ranges[i][1];
            Object result = parse.invoke(null, buffers[i], offset, length);
            System.out.println("switch4 slice=" + new String(buffers[i])
                    + " offset=" + offset + " length=" + length + " result=" + result);
        }
    }

    private static void testSwitchSimple(Class<?> type) throws Exception {
        Method test = type.getMethod("test", int.class);
        Object target = newInstance(type);
        int[] inputs = {
                Integer.MIN_VALUE, -5, -4, -1, 0, 1, 2, 3, 4, 5, 8, Integer.MAX_VALUE
        };
        for (int input : inputs) {
            System.out.println("simple.begin input=" + input);
            test.invoke(target, input);
            System.out.println("simple.end input=" + input);
        }
    }

    private static void testSwitchWithFallThroughCase2(Class<?> type) throws Exception {
        runCheck(type);
        Method test = type.getMethod("test", int.class, boolean.class, boolean.class);
        Object target = newInstance(type);
        int[] inputs = new int[17];
        for (int i = 0; i <= 14; i++) {
            inputs[i] = i - 5;
        }
        inputs[15] = Integer.MIN_VALUE;
        inputs[16] = Integer.MAX_VALUE;
        for (int input : inputs) {
            for (boolean b : new boolean[] {false, true}) {
                for (boolean c : new boolean[] {false, true}) {
                    Object result = test.invoke(target, input, b, c);
                    System.out.println("fall2 input=" + input + " b=" + b + " c=" + c
                            + " result=" + stringValue(result));
                }
            }
        }
    }

    private static void testSwitch2(Class<?> type) throws Exception {
        Method test = type.getMethod("test", int.class);
        Field[] booleans = new Field[SWITCH2_BOOLEAN_FIELDS.length];
        for (int i = 0; i < SWITCH2_BOOLEAN_FIELDS.length; i++) {
            booleans[i] = declaredField(type, SWITCH2_BOOLEAN_FIELDS[i]);
        }
        Field distance = declaredField(type, "multiTouchZoomOldDist");

        // Exercise all 2^5 initial boolean states. Float seeds and action positions rotate
        // across the sweep so every seed/key pair is represented without a full Cartesian
        // product. Every isolated action starts from a fresh object, preserving its setup.
        for (int mask = 0; mask < (1 << SWITCH2_BOOLEAN_FIELDS.length); mask++) {
            for (int actionIndex = 0; actionIndex < SWITCH2_ACTIONS.length; actionIndex++) {
                int seedIndex = (mask + actionIndex * 5) % SWITCH2_FLOAT_SEEDS.length;
                Object target = newSwitch2State(type, booleans, distance, mask,
                        SWITCH2_FLOAT_SEEDS[seedIndex]);
                int action = SWITCH2_ACTIONS[actionIndex];
                test.invoke(target, action);
                dumpSwitch2("switch2 isolated mask=" + mask + " seed=" + seedIndex
                        + " action=" + action, target, booleans, distance);
            }

            int sequenceSeed = (mask + 3) % SWITCH2_FLOAT_SEEDS.length;
            Object target = newSwitch2State(type, booleans, distance, mask,
                    SWITCH2_FLOAT_SEEDS[sequenceSeed]);
            dumpSwitch2("switch2 sequence-start mask=" + mask + " seed=" + sequenceSeed,
                    target, booleans, distance);
            for (int action : SWITCH2_SEQUENCE) {
                test.invoke(target, action);
                dumpSwitch2("switch2 sequence mask=" + mask + " action=" + action,
                        target, booleans, distance);
            }
        }
    }

    private static Object newSwitch2State(
            Class<?> type,
            Field[] booleans,
            Field distance,
            int mask,
            float initialDistance) throws Exception {
        Object target = newInstance(type);
        for (int i = 0; i < booleans.length; i++) {
            booleans[i].setBoolean(target, (mask & (1 << i)) != 0);
        }
        distance.setFloat(target, initialDistance);
        return target;
    }

    private static void dumpSwitch2(String prefix, Object target, Field[] booleans, Field distance)
            throws Exception {
        StringBuilder row = new StringBuilder(prefix).append(" state={");
        for (int i = 0; i < booleans.length; i++) {
            if (i != 0) {
                row.append(',');
            }
            row.append(booleans[i].getName()).append('=')
                    .append(booleans[i].getBoolean(target));
        }
        float value = distance.getFloat(target);
        row.append(",multiTouchZoomOldDist=").append(Float.toString(value))
                .append("/0x")
                .append(String.format(Locale.ROOT, "%08x", Float.floatToRawIntBits(value)))
                .append('}');
        System.out.println(row);
    }

    private static String stringValue(Object value) {
        if (value == null) {
            return "null";
        }
        String text = (String) value;
        StringBuilder escaped = new StringBuilder("\"");
        for (int i = 0; i < text.length(); i++) {
            char c = text.charAt(i);
            switch (c) {
                case '\\':
                    escaped.append("\\\\");
                    break;
                case '"':
                    escaped.append("\\\"");
                    break;
                case '\n':
                    escaped.append("\\n");
                    break;
                case '\r':
                    escaped.append("\\r");
                    break;
                case '\t':
                    escaped.append("\\t");
                    break;
                default:
                    escaped.append(c);
                    break;
            }
        }
        return escaped.append('"').toString();
    }
}

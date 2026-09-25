public final class ReferenceCastProbe {
    static int calls;

    public static String direct(Object value) {
        return (String) value;
    }

    public static int receiver(Object value) {
        return ((String) value).length();
    }

    public static int intArrayRead(Object value, int index) {
        return ((int[]) value)[index];
    }

    public static String[] stringArray(Object value) {
        return (String[]) value;
    }

    public static String[][] multiArray(Object value) {
        return (String[][]) value;
    }

    public static String local(Object value) {
        String result = (String) value;
        return result;
    }

    public static String staticParameter(Object value) {
        return consume((String) value);
    }

    public static String callOnce() {
        return (String) source();
    }

    public static Number nested(Object value) {
        return (Number) (Runnable) value;
    }

    public static boolean nestedCall() {
        return ((Number) (Runnable) source()) instanceof Comparable;
    }

    public static Object discardedCastBeforeCall(Object value) {
        String unused = (String) value;
        return source();
    }

    private static String consume(String value) {
        calls++;
        return value;
    }

    private static Object source() {
        calls++;
        return "source";
    }
}

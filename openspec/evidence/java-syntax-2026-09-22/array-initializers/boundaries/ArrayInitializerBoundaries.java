public final class ArrayInitializerBoundaries {
    private static final StringBuilder TRACE = new StringBuilder();
    private static int[] escaped;
    private static int escapedFirstAtPublication;

    private ArrayInitializerBoundaries() {}

    private static int mark(char c, int value) {
        TRACE.append(c);
        return value;
    }

    private static void escape(int[] value) {
        escaped = value;
        escapedFirstAtPublication = value[0];
    }

    public static int[] exactOrderedStores() {
        return new int[] { 11, 22, 33 };
    }

    public static int[] exactEffectfulStores() {
        TRACE.setLength(0);
        int[] value = new int[] { mark('a', 11), mark('b', 22), mark('c', 33) };
        System.out.println("effects=" + TRACE);
        return value;
    }

    public static String[] exactReferenceStores() {
        return new String[] { "left", null, "right" };
    }

    public static int[] dynamicLength(int n) {
        int[] value = new int[n];
        value[0] = 11;
        value[1] = 22;
        return value;
    }

    public static int[] duplicateIndex() {
        int[] value = new int[2];
        value[0] = 11;
        value[0] = 22;
        return value;
    }

    public static int[] skippedIndex() {
        int[] value = new int[3];
        value[1] = 22;
        value[2] = 33;
        return value;
    }

    public static int[] escapedBeforeStores() {
        int[] value = new int[2];
        escape(value);
        value[0] = 11;
        value[1] = 22;
        return value;
    }

    public static int escapedFirstAtPublication() {
        return escapedFirstAtPublication;
    }

    public static int[] storesAcrossBranch(boolean first) {
        int[] value = new int[2];
        if (first) {
            value[0] = 11;
        } else {
            value[0] = 22;
        }
        value[1] = 33;
        return value;
    }

    public static Object[] covariantAastoreThrows() {
        Object[] value = new String[1];
        value[0] = new Object();
        return value;
    }

    public static int[] runTime() {
        return exactOrderedStores();
    }
}

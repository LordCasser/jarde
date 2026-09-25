/** Source-only Java 8 control for assignment operand order and abrupt completion. */
public final class NarrowArrayStoreOrder {
    private static final StringBuilder TRACE = new StringBuilder();
    private static int arrayCalls;
    private static int indexCalls;
    private static int valueCalls;

    private NarrowArrayStoreOrder() {}

    public static void reset() {
        TRACE.setLength(0);
        arrayCalls = 0;
        indexCalls = 0;
        valueCalls = 0;
    }

    public static String result() {
        return TRACE + ":" + arrayCalls + "," + indexCalls + "," + valueCalls;
    }

    private static int[] array(int[] value, boolean fail) {
        TRACE.append('A');
        arrayCalls++;
        if (fail) throw new IllegalArgumentException("array");
        return value;
    }

    private static int index(int value, boolean fail) {
        TRACE.append('I');
        indexCalls++;
        if (fail) throw new IllegalArgumentException("index");
        return value;
    }

    private static int value(int value, boolean fail) {
        TRACE.append('V');
        valueCalls++;
        if (fail) throw new IllegalStateException("value");
        return value;
    }

    public static void store(int[] array, int index, int value,
            boolean failArray, boolean failIndex, boolean failValue) {
        array(array, failArray)[index(index, failIndex)] = value(value, failValue);
    }
}

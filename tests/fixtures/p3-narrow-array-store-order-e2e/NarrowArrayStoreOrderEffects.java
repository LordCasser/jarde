/** Source-only helpers make each assignment operand and each abrupt completion observable. */
public final class NarrowArrayStoreOrderEffects {
    public static final StringBuilder TRACE = new StringBuilder();
    public static int arrayCalls;
    public static int indexCalls;
    public static int valueCalls;

    private NarrowArrayStoreOrderEffects() {}

    public static void reset() {
        TRACE.setLength(0);
        arrayCalls = indexCalls = valueCalls = 0;
    }

    public static void mark(char mark) { TRACE.append(mark); }

    public static String state() {
        return TRACE + ":" + arrayCalls + "," + indexCalls + "," + valueCalls;
    }

    private static void array(boolean fail) {
        TRACE.append('A');
        arrayCalls++;
        if (fail) throw new IllegalArgumentException("array producer");
    }

    private static void index(boolean fail) {
        TRACE.append('I');
        indexCalls++;
        if (fail) throw new IllegalArgumentException("index producer");
    }

    public static int value(int value, boolean fail) {
        TRACE.append('V');
        valueCalls++;
        if (fail) throw new IllegalStateException("value producer");
        return value;
    }

    public static int index(int value, boolean fail) {
        index(fail);
        return value;
    }

    public static int[] byteArray(int[] value, boolean fail) {
        array(fail);
        return value;
    }

    public static int[] charArray(int[] value, boolean fail) {
        array(fail);
        return value;
    }

    public static int[] shortArray(int[] value, boolean fail) {
        array(fail);
        return value;
    }

    public static byte[] byteArray(byte[] value, boolean fail) {
        array(fail);
        return value;
    }

    public static char[] charArray(char[] value, boolean fail) {
        array(fail);
        return value;
    }

    public static short[] shortArray(short[] value, boolean fail) {
        array(fail);
        return value;
    }
}

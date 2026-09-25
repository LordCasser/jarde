public class ArrayInitializerProbe {
    private static final StringBuilder trace = new StringBuilder();

    private static void append(char value) {
        trace.append(value);
    }

    private static int intElement(int mode, int index) {
        append((char) ('a' + index));
        return index * 10 + 10 / (index - mode);
    }

    private static String stringElement(int mode, int index) {
        append((char) ('A' + index));
        return "s" + index + (10 / (index - mode));
    }

    public static int[] literalInts() {
        return new int[] {1, 2, 3, -4};
    }

    public static int[] emptyInts() {
        return new int[] {};
    }

    public static String[] literalStrings() {
        return new String[] {"left", null, "right"};
    }

    public static String[] emptyStrings() {
        return new String[] {};
    }

    public static int[] effectfulInts(int mode) {
        return new int[] {intElement(mode, 0), intElement(mode, 1), intElement(mode, 2)};
    }

    public static String[] effectfulStrings(int mode) {
        return new String[] {stringElement(mode, 0), stringElement(mode, 1), stringElement(mode, 2)};
    }

    public static String trace() {
        return trace.toString();
    }

    public static void reset() {
        trace.setLength(0);
    }
}

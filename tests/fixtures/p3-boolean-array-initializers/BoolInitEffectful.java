final class BoolInitEffectful {
    private static final StringBuilder trace = new StringBuilder();

    private static int element(int mode, int index) {
        trace.append((char) ('a' + index));
        return index * 2 + 10 / (index - mode);
    }

    static int[] values(int mode) {
        return new int[] {element(mode, 0), element(mode, 1), element(mode, 2)};
    }

    static String trace() {
        return trace.toString();
    }

    static void reset() {
        trace.setLength(0);
    }
}

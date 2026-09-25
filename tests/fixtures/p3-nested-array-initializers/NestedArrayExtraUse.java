public final class NestedArrayExtraUse {
    static int trace;

    static int element(int n) {
        trace = trace * 10 + n;
        return n;
    }

    static int[][] build() {
        int[][] result = new int[1][];
        int[] child = new int[] { element(4) };
        int length = child.length;
        result[0] = child;
        if (length != 1) throw new AssertionError();
        return result;
    }
}

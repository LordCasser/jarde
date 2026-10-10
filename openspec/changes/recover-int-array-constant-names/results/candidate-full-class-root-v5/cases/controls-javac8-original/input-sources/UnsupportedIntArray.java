public class UnsupportedIntArray {
    static final int VALUE = 7;

    static int make() {
        return 7;
    }

    static int[][] nested() {
        return new int[][] { { 7 } };
    }

    static int[] unsupportedLeaves(long source, int offset) {
        return new int[] { make(), (int) source, offset + 7 };
    }
}

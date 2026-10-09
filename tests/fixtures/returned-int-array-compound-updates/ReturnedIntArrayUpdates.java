public final class ReturnedIntArrayUpdates {
    static int trace;

    public static int plain2(int[][] a, int i, int j, int x) {
        return a[i][j] += x;
    }

    public static int plain3(int[][][] a, int i, int j, int k, int x) {
        return a[i][j][k] += x;
    }

    public static int scalar(int[] a, int i, int x) {
        return a[i] += x;
    }

    public static int traced(int[][] a, int r, int i, int x) {
        return a[row(r)][index(i)] += rhs(x);
    }

    static int row(int value) {
        trace = trace * 10 + 1;
        return value;
    }

    static int index(int value) {
        trace = trace * 10 + 2;
        return value;
    }

    static int rhs(int value) {
        trace = trace * 10 + 3;
        return value;
    }

    public static int swap(int[][] a) {
        a[0] = new int[] { 100 };
        return 7;
    }

    public static int replaceRow(int[][] a) {
        return a[0][0] += swap(a);
    }

    public static void different(int[][] a, int x) {
        a[0][0] = a[1][0] + x;
    }
}

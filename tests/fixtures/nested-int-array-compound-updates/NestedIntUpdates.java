public final class NestedIntUpdates {
    static int trace;

    public static void plain2(int[][] a, int i, int j, int x) {
        a[i][j] += x;
    }

    public static void plain3(int[][][] a, int i, int j, int k, int x) {
        a[i][j][k] += x;
    }

    public static void scalar(int[] a, int i, int x) {
        a[i] += x;
    }

    public static void traced(int[][] a, int r, int i, int x) {
        a[row(r)][index(i)] += rhs(x);
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
        a[0][0] += swap(a);
        return 7;
    }

    public static void different(int[][] a, int x) {
        a[0][0] = a[1][0] + x;
    }
}

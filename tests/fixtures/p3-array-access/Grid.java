public class Grid {
    static int created(int n, int i) {
        int[] a = new int[n];
        a[0] = 1;
        return a[i];
    }

    static int pick(int[] a, int i) {
        return a[i];
    }

    static int at(int[][] g, int i, int j) {
        return g[i][j];
    }

    static boolean flag() {
        boolean[] b = new boolean[3];
        return b[0];
    }

    static boolean test(boolean[] a, int i) {
        return a[i];
    }

    static char letter(char[] cs, int i) {
        return cs[i];
    }

    static int code(char[] cs, int i) {
        return cs[i];
    }

    static int first(byte[] a, int i) {
        return a[i];
    }

    static long wide(long[] a, int i) {
        return a[i];
    }

    static float real(float[] a, int i) {
        return a[i];
    }

    static double precise(double[] a, int i) {
        return a[i];
    }

    static short small(short[] a, int i) {
        return a[i];
    }

    static int size(int[] a) {
        return a.length;
    }

    static int put(int[] a, int i, int v) {
        a[i] = v;
        return a[i];
    }

    static void setByte(byte[] a, int i, byte v) {
        a[i] = v;
    }

    static String[] strings(int n) {
        return new String[n];
    }

    static String[] box(String value) {
        String[] a = new String[1];
        a[0] = value;
        return a;
    }

    static int cell(int n, int m) {
        int[][] g = new int[n][m];
        g[0][0] = 7;
        return g[0][0];
    }

    static int[] literal() {
        return new int[]{1, 2, 3};
    }

    static int[] chained(int n) {
        int[] a = new int[]{n};
        return a;
    }
}

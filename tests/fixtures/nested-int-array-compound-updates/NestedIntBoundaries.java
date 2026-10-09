public class NestedIntBoundaries {
    public static int returned(int[][] a, int i, int j, int x) {
        return a[i][j] += x;
    }
    public static void merged(boolean choose, int[][] a, int[][] b, int x) {
        int[] row = choose ? a[0] : b[0];
        row[0] += x;
    }
}

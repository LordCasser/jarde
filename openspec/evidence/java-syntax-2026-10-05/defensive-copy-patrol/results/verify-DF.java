public class DF extends java.lang.Object {
    private final int[] data;

    DF(int[] arg1) {
        super();
        this.data = (int[]) arg1.clone();
        return;
    }

    int[] data() {
        return (int[]) this.data.clone();
    }

    int sum() {
        int local1;
        int[] local2;
        local1 = 0;
        local2 = this.data;
        for (int local5 : local2) {
            local1 = local1 + local5;
        }
        return local1;
    }

    DF grow(int arg1) {
        int[] local2 = java.util.Arrays.copyOf(this.data, this.data.length + arg1);
        return new DF(local2);
    }

    static void copyRow(int[][] arg0, int[] arg1, int arg2) {
        java.lang.System.arraycopy((java.lang.Object) arg0[arg2], 0, (java.lang.Object) arg1, 0, arg0[arg2].length);
        return;
    }

    public static void main(java.lang.String[] arg0) {
        local2[0] = 99;
        int[][] local3 = new int[][]{new int[]{7, 8, 9}, new int[]{4, 5, 6}};
        int[] local4 = new int[3];
        copyRow(local3, local4, 1);
        return;
    }
}

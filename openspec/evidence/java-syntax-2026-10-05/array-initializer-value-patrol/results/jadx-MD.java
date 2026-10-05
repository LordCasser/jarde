package defpackage;

/* JADX INFO: loaded from: MD.class */
public class MD {
    static int[][] jagged = {new int[]{1}, new int[]{2, 3}, new int[]{4, 5, 6}};
    static int[][] reg = new int[2][3];
    static int[][] partial = new int[2][];

    static int sumJag() {
        int i = 0;
        for (int[] iArr : jagged) {
            for (int i2 : iArr) {
                i += i2;
            }
        }
        return i;
    }

    static int regSet() {
        reg[1][2] = 9;
        return reg[1][2];
    }

    static int partSet() {
        partial[0] = new int[]{7};
        partial[1] = new int[2];
        return partial[0][0] + partial[1].length;
    }

    static int[][] mkJagged() {
        return new int[][]{new int[]{8}, new int[]{9, 10}};
    }

    public static void main(java.lang.String[] strArr) {
        java.lang.System.out.println("" + sumJag() + "/" + regSet() + "/" + partSet() + "/" + mkJagged()[1][1]);
    }
}

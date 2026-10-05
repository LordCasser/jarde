package defpackage;

/* JADX INFO: loaded from: NL.class */
public class NL {
    static java.lang.String matrix(int[][][] iArr) {
        java.lang.StringBuilder sb = new java.lang.StringBuilder();
        loop0: for (int i = 0; i < iArr.length; i++) {
            int i2 = 0;
            while (true) {
                if (i2 < iArr[i].length) {
                    for (int i3 = 0; i3 < iArr[i][i2].length; i3++) {
                        int i4 = iArr[i][i2][i3];
                        if (i4 < 0) {
                            break;
                        }
                        if (i4 == 99) {
                            break loop0;
                        }
                        if (i4 != 50) {
                            sb.append(i4).append(",");
                        }
                    }
                    sb.append("|");
                    i2++;
                } else {
                    sb.append(";");
                    break;
                }
            }
        }
        return sb.toString();
    }

    static int findMid(int[][] iArr, int i) {
        int i2 = 0;
        loop0: for (int i3 = 0; i3 < iArr.length; i3++) {
            for (int i4 = 0; i4 < iArr[i3].length; i4++) {
                if (iArr[i3][i4] == i) {
                    i2++;
                    break;
                }
                if (iArr[i3][i4] < 0) {
                    break loop0;
                }
            }
        }
        return i2;
    }

    public static void main(java.lang.String[] strArr) {
        java.lang.System.out.println(matrix(new int[][][]{new int[][]{new int[]{1, 2}, new int[]{3, -1, 4}}, new int[][]{new int[]{5, 50, 6}, new int[]{7, 99, 8}}, new int[][]{new int[]{9}}}) + "/" + findMid(new int[][]{new int[]{1, 2}, new int[]{3, 2, 5}, new int[]{-1, 9}}, 2));
    }
}

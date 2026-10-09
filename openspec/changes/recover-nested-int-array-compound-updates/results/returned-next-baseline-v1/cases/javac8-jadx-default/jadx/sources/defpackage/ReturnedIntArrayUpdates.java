package defpackage;

/* JADX INFO: loaded from: ReturnedIntArrayUpdates.class */
public final class ReturnedIntArrayUpdates {
    static int trace;

    public static int plain2(int[][] iArr, int i, int i2, int i3) {
        int[] iArr2 = iArr[i];
        int i4 = iArr2[i2] + i3;
        iArr2[i2] = i4;
        return i4;
    }

    public static int plain3(int[][][] iArr, int i, int i2, int i3, int i4) {
        int[] iArr2 = iArr[i][i2];
        int i5 = iArr2[i3] + i4;
        iArr2[i3] = i5;
        return i5;
    }

    public static int scalar(int[] iArr, int i, int i2) {
        int i3 = iArr[i] + i2;
        iArr[i] = i3;
        return i3;
    }

    public static int traced(int[][] iArr, int i, int i2, int i3) {
        int[] iArr2 = iArr[row(i)];
        int iIndex = index(i2);
        int iRhs = iArr2[iIndex] + rhs(i3);
        iArr2[iIndex] = iRhs;
        return iRhs;
    }

    static int row(int i) {
        trace = (trace * 10) + 1;
        return i;
    }

    static int index(int i) {
        trace = (trace * 10) + 2;
        return i;
    }

    static int rhs(int i) {
        trace = (trace * 10) + 3;
        return i;
    }

    public static int swap(int[][] iArr) {
        iArr[0] = new int[]{100};
        return 7;
    }

    public static int replaceRow(int[][] iArr) {
        int[] iArr2 = iArr[0];
        int iSwap = iArr2[0] + swap(iArr);
        iArr2[0] = iSwap;
        return iSwap;
    }

    public static void different(int[][] iArr, int i) {
        iArr[0][0] = iArr[1][0] + i;
    }
}

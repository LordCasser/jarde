package defpackage;

/* JADX INFO: loaded from: NestedIntUpdates.class */
public final class NestedIntUpdates {
    static int trace;

    public static void plain2(int[][] iArr, int i, int i2, int i3) {
        int[] iArr2 = iArr[i];
        iArr2[i2] = iArr2[i2] + i3;
    }

    public static void plain3(int[][][] iArr, int i, int i2, int i3, int i4) {
        int[] iArr2 = iArr[i][i2];
        iArr2[i3] = iArr2[i3] + i4;
    }

    public static void scalar(int[] iArr, int i, int i2) {
        iArr[i] = iArr[i] + i2;
    }

    public static void traced(int[][] iArr, int i, int i2, int i3) {
        int[] iArr2 = iArr[row(i)];
        int iIndex = index(i2);
        iArr2[iIndex] = iArr2[iIndex] + rhs(i3);
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
        iArr2[0] = iArr2[0] + swap(iArr);
        return 7;
    }

    public static void different(int[][] iArr, int i) {
        iArr[0][0] = iArr[1][0] + i;
    }
}

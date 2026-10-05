package defpackage;

/* JADX INFO: loaded from: SW.class */
public class SW {
    static int a = 1;
    static int b = 2;

    static void swapFields() {
        int i = a;
        a = b;
        b = i;
    }

    static int[] swapArr(int[] iArr) {
        int i = iArr[0];
        iArr[0] = iArr[1];
        iArr[1] = i;
        return iArr;
    }

    static int[] manualCopy(int[] iArr) {
        int[] iArr2 = new int[iArr.length];
        for (int i = 0; i < iArr.length; i++) {
            iArr2[i] = iArr[i];
        }
        return iArr2;
    }

    static int sum2d(int[][] iArr) {
        int i = 0;
        for (int i2 = 0; i2 < iArr.length; i2++) {
            for (int i3 = 0; i3 < iArr[i2].length; i3++) {
                i += iArr[i2][i3];
            }
        }
        return i;
    }

    public static void main(java.lang.String[] strArr) {
        swapFields();
        java.lang.System.out.println("" + a + "," + b + "/" + java.util.Arrays.toString(swapArr(new int[]{7, 9})) + "/" + java.util.Arrays.toString(manualCopy(new int[]{4, 5})) + "/" + sum2d(new int[][]{new int[]{1, 2}, new int[]{3, 4}}));
    }
}

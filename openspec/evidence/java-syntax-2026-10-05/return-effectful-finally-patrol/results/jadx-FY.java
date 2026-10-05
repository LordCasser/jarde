package defpackage;

/* JADX INFO: loaded from: FY.class */
public class FY {
    static int emptyFin(int[] iArr) {
        for (int i : iArr) {
            if (i == 3) {
                return i * 1000;
            }
        }
        return -1;
    }

    static int noCall(int[] iArr) {
        int i;
        int i2 = 0;
        for (int i3 : iArr) {
            if (i3 == 3) {
                try {
                    int i4 = i2 + 7;
                    return i4;
                } finally {
                    i = i2 + 100;
                }
            }
            i2 += 100;
        }
        return i2;
    }

    static int loopless(int[] iArr) {
        try {
            if (iArr.length <= 0 || iArr[0] != 3) {
                java.lang.System.out.println("f");
                return -1;
            }
            java.lang.System.out.println("f");
            return 3000;
        } catch (java.lang.Throwable th) {
            java.lang.System.out.println("f");
            throw th;
        }
    }

    public static void main(java.lang.String[] strArr) {
        java.lang.System.out.println("" + emptyFin(new int[]{1, 3}) + "/" + noCall(new int[]{1, 3}) + "/" + loopless(new int[]{3}));
    }
}

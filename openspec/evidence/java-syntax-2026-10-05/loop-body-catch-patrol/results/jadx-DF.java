package defpackage;

/* JADX INFO: loaded from: DF.class */
public class DF {
    static int defaultFirst(int i) {
        switch (i) {
            case 1:
                return 10;
            case 2:
                return 20;
            default:
                return 0;
        }
    }

    static int defaultFallthrough(int i) {
        switch (i) {
            case 1:
                return 100;
            case 2:
            default:
                return 200;
        }
    }

    static int loopCatch(int[] iArr) {
        int i = 0;
        for (int i2 : iArr) {
            if (i2 < 0) {
                throw new java.lang.IllegalArgumentException("neg");
            }
            try {
                i += i2;
            } catch (java.lang.IllegalArgumentException e) {
                i--;
            }
            i--;
        }
        return i;
    }

    static int loopFinally(int[] iArr) {
        int i = 0;
        for (int i2 : iArr) {
            if (i2 != 3) {
                i += i2;
            }
            try {
                i += 1000;
            } catch (java.lang.Throwable th) {
                int i3 = i + 1000;
                throw th;
            }
        }
        return i;
    }

    public static void main(java.lang.String[] strArr) {
        java.lang.System.out.println("" + defaultFirst(1) + "/" + defaultFirst(9) + "/" + defaultFallthrough(1) + "/" + defaultFallthrough(2) + "/" + defaultFallthrough(9) + "/" + loopCatch(new int[]{1, -2, 3}) + "/" + loopFinally(new int[]{3, 4}));
    }
}

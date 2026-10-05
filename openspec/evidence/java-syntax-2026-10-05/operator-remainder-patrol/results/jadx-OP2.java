package defpackage;

/* JADX INFO: loaded from: OP2.class */
public class OP2 {
    static int shl(int i) {
        return i << 2;
    }

    static int shr(int i) {
        return i >> 1;
    }

    static int ushr(int i) {
        return i >>> 1;
    }

    static long lshl(long j) {
        return j << 3;
    }

    static float nanf() {
        return Float.NaN;
    }

    static double pinf() {
        return Double.POSITIVE_INFINITY;
    }

    static double ninf() {
        return Double.NEGATIVE_INFINITY;
    }

    static float pzero() {
        return -0.0f;
    }

    static boolean condAssign(int i) {
        return i + 1 > 0;
    }

    static int condAssignOld(int i) {
        int i2 = i + 1;
        if (i2 > 0 && i2 > 0) {
            return i2;
        }
        return -1;
    }

    public static void main(java.lang.String[] strArr) {
        java.lang.System.out.println("" + shl(3) + "/" + shr(-8) + "/" + ushr(-8) + "/" + lshl(2L) + "/" + nanf() + "/" + pinf() + "/" + ninf() + "/" + (pzero() == 0.0f) + "/" + condAssign(0) + "/" + condAssignOld(0));
    }
}

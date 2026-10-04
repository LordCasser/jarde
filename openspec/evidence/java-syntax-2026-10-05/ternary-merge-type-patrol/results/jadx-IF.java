package defpackage;

/* JADX INFO: loaded from: IF.class */
public class IF {
    static int dense(boolean z, boolean z2, int i) {
        if (z) {
            return z2 ? i + 1 : i - 1;
        }
        return i * 2;
    }

    static int chain(int i) {
        if (i < 0) {
            return -1;
        }
        if (i == 0) {
            return 0;
        }
        if (i > 9) {
            return 9;
        }
        return i;
    }

    static boolean sc(boolean z, boolean z2) {
        return (z && z2) || !(z || z2);
    }

    static int guard(int i) {
        if (i <= 0 || i >= 10) {
            return (i < 10 || i >= 100) ? 0 : 2;
        }
        return 1;
    }

    static java.lang.Object poly(boolean z) {
        if (z) {
            return 1;
        }
        return "s";
    }

    public static void main(java.lang.String[] strArr) {
        java.lang.System.out.println("" + dense(true, false, 5) + "/" + chain(15) + "/" + sc(true, true) + "/" + guard(50) + "/" + poly(false));
    }
}

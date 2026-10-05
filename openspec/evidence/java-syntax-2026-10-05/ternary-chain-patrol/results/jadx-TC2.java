package defpackage;

/* JADX INFO: loaded from: TC2.class */
public class TC2 {
    static java.lang.String plainAnd(boolean z, boolean z2) {
        return (z && z2) ? "both" : "one";
    }

    static java.lang.String chain(boolean z, boolean z2) {
        if (z) {
            return "x";
        }
        return z2 ? "y" : "n";
    }

    static java.lang.String shortOnly(boolean z, boolean z2) {
        if (z && z2) {
            return "1";
        }
        return (z || z2) ? "2" : "3";
    }

    static java.lang.String andOr(boolean z, boolean z2, boolean z3) {
        return ((z || z2) && z3) ? "y" : "n";
    }

    public static void main(java.lang.String[] strArr) {
        java.lang.System.out.println("" + plainAnd(true, true) + "/" + chain(false, false) + "/" + andOr(false, true, true));
    }
}

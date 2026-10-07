package defpackage;

/* JADX INFO: loaded from: ScopePlan.class */
public final class ScopePlan {
    private ScopePlan() {
    }

    static int catchOnly(boolean z) {
        if (!z) {
            return 1;
        }
        try {
            throw null;
        } catch (java.lang.NullPointerException e) {
            return 2;
        }
    }

    static int assignedAcrossTry(boolean z) {
        int i;
        if (z) {
            try {
                throw null;
            } catch (java.lang.NullPointerException e) {
                i = 4;
            }
        } else {
            i = 3;
        }
        return i;
    }

    static int assignedAcrossIf(boolean z) {
        return z ? 5 : 6;
    }

    static int nestedHandlerOnly(java.lang.Runnable runnable) {
        try {
            runnable.run();
            return 0;
        } catch (java.lang.IllegalArgumentException e) {
            return 8;
        } catch (java.lang.RuntimeException e2) {
            return 9;
        }
    }

    static int nestedAcross(boolean z) {
        int i;
        if (z) {
            try {
                throw null;
            } catch (java.lang.NullPointerException e) {
                i = 11;
            } catch (java.lang.RuntimeException e2) {
                i = 12;
            }
        } else {
            i = 10;
        }
        return i;
    }

    public static void main(java.lang.String[] strArr) {
        java.lang.System.out.print(catchOnly(false) + "," + catchOnly(true) + ",");
        java.lang.System.out.print(assignedAcrossTry(false) + "," + assignedAcrossTry(true) + ",");
        java.lang.System.out.print(assignedAcrossIf(false) + "," + assignedAcrossIf(true) + ",");
        java.lang.System.out.print(nestedHandlerOnly((java.lang.Runnable) null) + ",");
        java.lang.System.out.print(nestedAcross(false) + "," + nestedAcross(true) + "\n");
    }
}

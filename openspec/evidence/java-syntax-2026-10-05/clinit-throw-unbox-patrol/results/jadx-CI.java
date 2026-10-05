package defpackage;

/* JADX INFO: loaded from: CI.class */
public class CI {
    static boolean ready = false;
    static final int VALUE = boot();

    static int boot() {
        if (ready) {
            return 42;
        }
        throw new java.lang.IllegalStateException("not ready");
    }

    static int safeGet() {
        try {
            java.lang.Class.forName("CI");
        } catch (java.lang.Throwable th) {
        }
        return VALUE;
    }

    static boolean unbox(java.lang.Boolean bool) {
        return bool.booleanValue();
    }

    static int unboxIf(java.lang.Boolean bool) {
        return bool.booleanValue() ? 1 : 0;
    }

    static int doAnd(int i) {
        int i2 = 0;
        do {
            i2++;
            if (i2 >= i) {
                break;
            }
        } while (i2 < 100);
        return i2;
    }

    static int whileOr(java.util.List<java.lang.Integer> list) {
        int iIntValue = 0;
        int i = 0;
        while (true) {
            if (i >= list.size() && iIntValue >= 0) {
                return iIntValue;
            }
            if (i < list.size()) {
                iIntValue += list.get(i).intValue();
            }
            i++;
        }
    }

    public static void main(java.lang.String[] strArr) {
        java.lang.System.out.println("" + safeGet() + "/" + unbox(java.lang.Boolean.TRUE) + "/" + unboxIf(java.lang.Boolean.FALSE) + "/" + doAnd(5) + "/" + whileOr(java.util.Arrays.asList(1, 2, 3)));
    }
}

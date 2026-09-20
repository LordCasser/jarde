public class BooleanContexts {
    public static boolean staticFlag = true;

    public BooleanContexts() {
    }

    // --- the two defect shapes (P3: a boolean context types by the descriptor) -------------

    public static boolean isZero(int x) {
        if (x == 0) {
            return true;
        }
        return false;
    }

    public static boolean flag() {
        return true;
    }

    public static int parity(int x) {
        if (flag()) {
            return 1;
        }
        return 0;
    }

    // --- the evidence shapes a boolean context must keep presenting (P3-R5's proof) --------

    public static boolean passed(boolean b) {
        return b;
    }

    public static boolean callFlag() {
        return flag();
    }

    public static boolean fieldFlag() {
        return staticFlag;
    }

    // --- the shapes a local's own declaration types (the third site of the same family) ----

    public static boolean localFromCall() {
        boolean c = flag();
        return c;
    }

    public static boolean pick(int n, boolean a, boolean b) {
        boolean c;
        if (n != 0) {
            c = a;
        } else {
            c = b;
        }
        return c;
    }

    public static boolean fromLocal(boolean b) {
        boolean c = b;
        boolean d = c;
        return d;
    }

    public static boolean assignFromCall(boolean b) {
        b = flag();
        return b;
    }

    public static boolean throughLocal(boolean b) {
        boolean c = b;
        return c;
    }

    public static boolean negated(boolean b) {
        return !b;
    }

    public static int staticFlagCount() {
        if (staticFlag) {
            return 1;
        }
        return 0;
    }

    // --- the int-shaped controls (must not be re-typed) -----------------------------------

    public static int intLocal(int n) {
        int x = 0;
        if (n != 0) {
            x = 1;
        } else {
            x = n;
        }
        return x;
    }

    public static int count(boolean b) {
        if (b) {
            return 1;
        }
        return 0;
    }

    public static int nonzero(int x) {
        if (x != 0) {
            return 1;
        }
        return 0;
    }

    public static int answer() {
        return 1;
    }
}

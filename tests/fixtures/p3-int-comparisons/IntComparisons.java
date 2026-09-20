public class IntComparisons {
    // --- the three defect shapes: an integer binary comparison with the constant on the left ---

    public static int oneFirst(int n) {
        if (1 == n) {
            return 3;
        }
        return 4;
    }

    public static int zeroFirst(int n) {
        if (0 < n) {
            return 3;
        }
        return 4;
    }

    public static int oneLess(int n) {
        if (1 < n) {
            return 3;
        }
        return 4;
    }

    // --- the same comparison forms with the constant on the right -----------------------------

    public static int oneLast(int n) {
        if (n == 1) {
            return 3;
        }
        return 4;
    }

    public static int zeroLast(int n) {
        if (n > 0) {
            return 3;
        }
        return 4;
    }

    // --- the int control with two variable operands (the shape that must not be read as boolean) ---

    public static int nonzero(int n) {
        if (n != 0) {
            return 3;
        }
        return 4;
    }

    // --- the predecessor change's boolean shapes, kept here as the same sample's controls ------

    public static boolean isZero(int x) {
        if (x == 0) {
            return true;
        }
        return false;
    }

    public static int count(boolean b) {
        if (b) {
            return 1;
        }
        return 0;
    }

    public static boolean throughLocal(boolean b) {
        boolean c = b;
        return c;
    }
}

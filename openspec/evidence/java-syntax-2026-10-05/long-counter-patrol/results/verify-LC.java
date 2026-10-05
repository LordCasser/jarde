public class LC extends java.lang.Object {
    public LC() {
        super();
        return;
    }

    static long longSum(long arg0) {
        long local2;
        long local4;
        local2 = 0L;
        local4 = 1L;
        while (local4 <= arg0) {
            local2 = local2 + local4;
            local4 = local4 + 1L;
        }
        return local2;
    }

    static int longCmp(long arg0, long arg2, int arg4) {
        return arg0 < arg2 ? arg4 : arg0 > arg2 ? -arg4 : 0;
    }

    static double floatStep(int arg0) {
        double local1;
        int local3;
        local1 = 0x0.0000000000000p-1022d;
        for (local3 = 0; local3 < arg0; local3 = local3 + 1) {
            local1 = local1 + 0x1.0000000000000p0d / (double) (local3 + 1);
        }
        return local1;
    }

    static int narrow(long arg0) {
        int local2;
        local2 = 0;
        if (arg0 > 2147483647L) {
            local2 = 2147483647;
        } else if (arg0 < -2147483648L) {
            local2 = -2147483648;
    } else {
            local2 = (int) arg0;
    }
        return local2;
    }

    public static void main(java.lang.String[] arg0) {
        java.lang.System.out.println("" + longSum(100L) + "/" + longCmp(5L, 3L, 2) + "/" + longCmp(3L, 5L, 2) + "/" + longCmp(4L, 4L, 2) + "/" + floatStep(4) + "/" + narrow(5000000000L) + "/" + narrow(42L));
        return;
    }
}

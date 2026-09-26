package defpackage;

import java.nio.file.FileSystemException;
import java.util.List;

/* JADX INFO: loaded from: Patrol2.class */
public class Patrol2 {
    static int calls;
    static final /* synthetic */ boolean $assertionsDisabled;

    static {
        $assertionsDisabled = !Patrol2.class.desiredAssertionStatus();
    }

    static int check(int arg0) {
        if ($assertionsDisabled || arg0 > 0) {
            return arg0 * 2;
        }
        throw new AssertionError("bad " + arg0);
    }

    static synchronized int bump(int arg0) {
        return arg0 + 1;
    }

    static int dow(int arg0) {
        int n = 0;
        do {
            n++;
            if (n % 2 != 0) {
                n += 5;
            }
        } while (n < arg0);
        return n;
    }

    static int strSwitch(String s) {
        switch (s) {
            case "a":
                return 1;
            case "b":
                return 2;
            default:
                return -1;
        }
    }

    static int cond(int arg0) {
        int i;
        int r = arg0 > 0 ? side(1) : side(2);
        if (arg0 > 10) {
            i = arg0 > 100 ? 3 : 2;
        } else {
            i = 1;
        }
        int t = i;
        return r + t;
    }

    static int side(int x) {
        calls++;
        return x;
    }

    static int bits(int a, int b, int c) {
        return (a & b) | (c ^ 255) | (a >>> 3) | (a << 1);
    }

    static boolean fp(double a, double b) {
        return a > b || a <= b;
    }

    static int lng(long a) {
        long m = a * 4294967296L;
        if (m == -1) {
            return 1;
        }
        if (m > 4611686018427387903L) {
            return 2;
        }
        return (int) (m >>> 32);
    }

    static int multiCatch(int arg0) {
        try {
            if (arg0 == 0) {
                throw new NumberFormatException("n");
            }
            if (arg0 == 1) {
                throw new FileSystemException("f");
            }
            return 0;
        } catch (NumberFormatException | FileSystemException e) {
            return e.getMessage() == null ? 1 : 2;
        }
    }

    static int foreach(int[] arr, List<String> list) {
        int n = 0;
        for (int x : arr) {
            if (x >= 0) {
                n += x;
            }
        }
        for (String s : list) {
            if (s.isEmpty()) {
                break;
            }
            n++;
        }
        return n;
    }
}

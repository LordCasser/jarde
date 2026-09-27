package cf03;

/* JADX INFO: loaded from: input.jar:cf03/BranchShapes.class */
public class BranchShapes {
    public static int hits;

    private static boolean matches(String str, String str2) {
        hits++;
        return str.equals(str2);
    }

    public static int chain(String str) {
        int i;
        hits = 0;
        if (matches(str, "a")) {
            i = 1;
        } else if (matches(str, "b")) {
            i = 2;
        } else if (matches(str, "3")) {
            i = 3;
        } else if (matches(str, "$")) {
            i = 4;
        } else {
            i = -1;
            hits += 10;
        }
        return Math.abs(i * 10);
    }

    public static boolean nested(boolean z, int i, int i2) {
        if (z) {
            if (i == 0 || i2 == 0) {
                return false;
            }
        } else if (i == 0 || i2 == 0) {
            return false;
        }
        hits++;
        return true;
    }

    public static int guards(String str) {
        if (str == null) {
            return -1;
        }
        if (str.length() != 1) {
            return -2;
        }
        char cCharAt = str.charAt(0);
        if (cCharAt == 'a') {
            return 1;
        }
        return cCharAt == 'b' ? 2 : 0;
    }
}

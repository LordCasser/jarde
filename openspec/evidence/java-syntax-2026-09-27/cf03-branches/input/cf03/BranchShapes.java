package cf03;

public class BranchShapes {
    public static int hits;

    private static boolean matches(String value, String expected) {
        hits++;
        return value.equals(expected);
    }

    public static int chain(String value) {
        hits = 0;
        int result;
        if (matches(value, "a")) {
            result = 1;
        } else if (matches(value, "b")) {
            result = 2;
        } else if (matches(value, "3")) {
            result = 3;
        } else if (matches(value, "$")) {
            result = 4;
        } else {
            result = -1;
            hits += 10;
        }
        result *= 10;
        return Math.abs(result);
    }

    public static boolean nested(boolean gate, int x, int y) {
        if (gate) {
            if (x == 0 || y == 0) {
                return false;
            }
        } else if (x == 0 || y == 0) {
            return false;
        }
        hits++;
        return true;
    }

    public static int guards(String value) {
        if (value == null) {
            return -1;
        }
        if (value.length() != 1) {
            return -2;
        }
        char c = value.charAt(0);
        if (c == 'a') {
            return 1;
        }
        if (c == 'b') {
            return 2;
        }
        return 0;
    }
}

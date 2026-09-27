package cf03;

public class ChainOnly {
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
}

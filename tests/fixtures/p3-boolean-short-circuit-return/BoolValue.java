public final class BoolValue {
    static int calls;

    static boolean positive(int n) {
        calls++;
        return n > 0;
    }

    public static boolean and(int a, int b) {
        return a > 0 && b > 0;
    }

    public static boolean or(int a, int b) {
        return a > 0 || b > 0;
    }

    public static boolean effectfulAnd(int a, int b) {
        return positive(a) && positive(b);
    }

    public static boolean effectfulOr(int a, int b) {
        return positive(a) || positive(b);
    }

    public static void main(String[] args) {
        for (int a = -1; a <= 1; a += 2) {
            for (int b = -1; b <= 1; b += 2) {
                calls = 0;
                System.out.println(a + "," + b + ":" + and(a, b) + "," + or(a, b)
                        + "," + effectfulAnd(a, b) + "," + effectfulOr(a, b) + "," + calls);
            }
        }
    }
}

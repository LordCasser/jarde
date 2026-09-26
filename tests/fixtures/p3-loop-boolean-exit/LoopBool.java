public final class LoopBool {
    public static int andWhile(int a, int b) {
        int n = 0;
        while (a > 0 && b > 0) {
            n += a;
            a--;
            b--;
        }
        return n;
    }

    public static int orWhile(int a, int b) {
        int n = 0;
        while (a > 0 || b > 0) {
            n++;
            a--;
            b--;
        }
        return n;
    }

    public static int mixedWhile(int a, int b) {
        int n = 0;
        while ((a > 0 && b > 0) || n < 2) {
            n++;
            a--;
            b--;
        }
        return n;
    }
}

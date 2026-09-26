public final class DoLoopBool {
    public static int andDo(int a, int b) {
        int n = 0;
        do {
            n++;
            a--;
            b--;
        } while (a > 0 && b > 0);
        return n;
    }

    public static int orDo(int a, int b) {
        int n = 0;
        do {
            n++;
            a--;
            b--;
        } while (a > 0 || b > 0);
        return n;
    }
}

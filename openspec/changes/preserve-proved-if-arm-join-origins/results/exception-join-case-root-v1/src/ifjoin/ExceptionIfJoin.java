package ifjoin;
public final class ExceptionIfJoin {
    static void mayThrow() {}
    public static int withException(int n) {
        int total = 0;
        int i = 0;
        try {
            while (i < n) {
                if ((i & 1) == 0) {
                    mayThrow();
                    total += 2;
                } else {
                    total++;
                }
                i++;
            }
        } catch (RuntimeException e) {
            return -1;
        }
        return total;
    }
}

public class Before {
    static int guarded(Object o) {
        int x = 5;
        if (o instanceof String) {
            return x + 1;
        }
        return x;
    }

    static int loopThenBreak(int n) {
        int x = 0;
        while (n > 0) {
            x = x + n;
            if (n == 5) {
                break;
            }
            n = n - 1;
        }
        return x;
    }
}

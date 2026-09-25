public class HandlerStore {
    static int two(int n) {
        try {
            if (n > 0) {
                n = n + 1;
            } else {
                n = n - 1;
            }
        } catch (RuntimeException e) {
            try {
                n = -1;
            } catch (IllegalArgumentException e2) {
                n = -2;
            }
        }
        return n;
    }
}

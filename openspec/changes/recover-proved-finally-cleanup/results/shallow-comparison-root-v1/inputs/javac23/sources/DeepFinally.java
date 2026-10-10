public class DeepFinally {
    static int trace;

    private static void cleanup() {
        trace++;
    }

    static int run(int x) {
        try {
            while (x > 2) {
                x--;
                while (x > 1) {
                    x--;
                }
            }
            return x;
        } finally {
            cleanup();
        }
    }
}

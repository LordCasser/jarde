public class BranchFinally {
    static final int DEPTH = 2;
    static int trace;

    private static void cleanup() {
        trace++;
    }

    static int run(int x) {
        try {
            if (trace + x > 1) {
                trace += 1;
                if (x > 2) {
                    trace += 2;
                } else {
                    trace -= 2;
                }
            } else {
                trace -= 1;
            }
            return x;
        } finally {
            cleanup();
        }
    }
}

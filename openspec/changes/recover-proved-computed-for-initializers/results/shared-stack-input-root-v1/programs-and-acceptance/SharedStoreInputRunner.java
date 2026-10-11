public final class SharedStoreInputRunner {
    private static void run(String label, int limit, boolean throwing,
                            int[] values, int target, int lower) {
        SharedStoreInput.savedField = -99;
        SharedStoreInput.limitCalls = 0;
        SharedStoreInput.limitValue = limit;
        SharedStoreInput.throwFromLimit = throwing;
        try {
            int result = SharedStoreInput.find(values, target, lower);
            System.out.println(label + ":return=" + result
                    + ",calls=" + SharedStoreInput.limitCalls
                    + ",saved=" + SharedStoreInput.savedField);
        } catch (IllegalStateException expected) {
            System.out.println(label + ":throw=" + expected.getMessage()
                    + ",calls=" + SharedStoreInput.limitCalls
                    + ",saved=" + SharedStoreInput.savedField);
        }
    }

    public static void main(String[] args) {
        run("hit", 3, false, new int[] {4, 8, 7, 2}, 7, 0);
        run("miss", 3, false, new int[] {4, 8, 7, 2}, 99, 0);
        run("below-lower", 3, false, new int[] {4, 8, 7, 2}, 7, 3);
        run("throw", 3, true, new int[] {4, 8, 7, 2}, 7, 0);
    }
}

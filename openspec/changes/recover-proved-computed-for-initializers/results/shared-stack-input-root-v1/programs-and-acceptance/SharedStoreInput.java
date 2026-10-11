final class SharedStoreInput {
    static int savedField;
    static int limitCalls;
    static int limitValue;
    static boolean throwFromLimit;

    static int limit() {
        limitCalls++;
        if (throwFromLimit) {
            throw new IllegalStateException("limit");
        }
        return limitValue;
    }

    static int find(int[] values, int target, int lower) {
        int i = (savedField = limit() - 1);
        while (i >= lower) {
            if (values[i] == target) {
                return i;
            }
            i--;
        }
        return -1;
    }
}

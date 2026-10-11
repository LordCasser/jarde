final class ComputedForDirectReturn {
    static int end;
    static int calls;

    static int limit() {
        calls++;
        return end;
    }

    static int find(int[] values, int target, int lower) {
        for (int i = limit() - 1; i >= lower; i--) {
            if (values[i] == target) return i;
        }
        return -1;
    }
}

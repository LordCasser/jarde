final class ArrayForeachRefusal {
    static int indexInBody(int[] values) {
        int sum = 0;
        int[] captured = values;
        int length = captured.length;
        for (int i = 0; i < length; i++) {
            int value = captured[i];
            sum += value + i;
        }
        return sum;
    }

    static int differentArrays(int[] limit, int[] values) {
        int sum = 0;
        int[] boundary = limit;
        int[] read = values;
        int length = boundary.length;
        for (int i = 0; i < length; i++) {
            int value = read[i];
            sum += value;
        }
        return sum;
    }

    static int indexAfterLoop(int[] values) {
        int sum = 0;
        int[] captured = values;
        int length = captured.length;
        int i;
        for (i = 0; i < length; i++) {
            int value = captured[i];
            sum += value;
        }
        return sum + i;
    }

    static int effectInBinding(int[] values) {
        int sum = 0;
        int[] captured = values;
        int length = captured.length;
        for (int i = 0; i < length; i++) {
            int value = captured[i] + tick();
            sum += value;
        }
        return sum;
    }

    static int escapedElement(int[] values) {
        int sum = 0;
        int element = 0;
        int[] captured = values;
        int length = captured.length;
        for (int i = 0; i < length; i++) {
            element = captured[i];
            sum += element;
        }
        return sum + element;
    }

    private static int tick() {
        return 1;
    }
}

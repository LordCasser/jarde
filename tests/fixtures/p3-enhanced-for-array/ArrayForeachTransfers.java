final class ArrayForeachTransfers {
    static int skipNegative(int[] values) {
        int sum = 0;
        int[] captured = values;
        int length = captured.length;
        outer: for (int i = 0; i < length; i++) {
            int element = captured[i];
            if (element < 0) {
                continue outer;
            }
            sum += element;
        }
        return sum;
    }

    static int stopAtNegative(int[] values) {
        int sum = 0;
        int[] captured = values;
        int length = captured.length;
        for (int i = 0; i < length; i++) {
            int element = captured[i];
            if (element < 0) {
                break;
            }
            sum += element;
        }
        return sum;
    }
}

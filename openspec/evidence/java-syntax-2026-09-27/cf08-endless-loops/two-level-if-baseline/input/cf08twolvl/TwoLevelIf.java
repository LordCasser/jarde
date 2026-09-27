package cf08twolvl;

public final class TwoLevelIf {
    private static int calls;

    private static int cost(int value) {
        calls++;
        return value;
    }

    static int calls() {
        return calls;
    }

    static void resetCalls() {
        calls = 0;
    }

    public static int pick(int[] xs) {
        int result;
        if (xs == null) {
            result = -1;
        } else {
            int length = xs.length;
            if (length == 0) {
                result = -1;
            } else {
                int i = 0;
                while (true) {
                    if (i >= length) {
                        result = cost(7);
                        break;
                    }
                    result = xs[i];
                    if (result == 3) {
                        break;
                    }
                    i++;
                }
            }
            result += calls;
        }
        return result;
    }
}

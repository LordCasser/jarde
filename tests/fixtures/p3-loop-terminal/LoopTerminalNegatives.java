package cf07;

public class LoopTerminalNegatives {
    public static int extraEntry(int[] array, int target, int start, int end) {
        int i = end - 1;
        if (start < 0) {
            return i;
        }
        while (i >= start) {
            if (array[i] == target) {
                return i;
            }
            i--;
        }
        return -1;
    }

    public static int sharedLeaf(int[] array, int target, int start, int end) {
        for (int i = end - 1; i >= start; i--) {
            if (array[i] == target || i == start + 1) {
                return i;
            }
        }
        return -1;
    }

    public static int nonterminalExit(int[] array, int target, int start, int end) {
        int result = -1;
        for (int i = end - 1; i >= start; i--) {
            if (array[i] == target) {
                result = i;
                break;
            }
        }
        return result;
    }

    public static int exceptionalLeaf(int[] array, int target, int start, int end) {
        try {
            for (int i = end - 1; i >= start; i--) {
                if (array[i] == target) {
                    return i;
                }
            }
        } catch (RuntimeException ignored) {
            return -2;
        }
        return -1;
    }

    public static int otherValue(int[] array, int target, int start, int end) {
        for (int i = end - 1; i >= start; i--) {
            if (array[i] == target) {
                return array[i];
            }
        }
        return -1;
    }
}

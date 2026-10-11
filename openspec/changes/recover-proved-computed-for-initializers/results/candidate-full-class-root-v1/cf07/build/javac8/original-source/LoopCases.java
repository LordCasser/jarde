package cf07;

public class LoopCases {
    public static int andWhile(boolean enabled) {
        int i = 0;
        while (enabled && i < 10) {
            i++;
        }
        return i;
    }

    public static int counted(int a, int b) {
        int c = a + b;
        for (int i = a; i < b; i++) {
            if (i == 7) {
                c += 2;
            } else {
                c *= 2;
            }
        }
        c--;
        return c;
    }

    public static int lastIndexOf(int[] array, int target, int start, int end) {
        for (int i = end - 1; i >= start; i--) {
            if (array[i] == target) {
                return i;
            }
        }
        return -1;
    }
}

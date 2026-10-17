public final class SwitchLoopExceptionBoundary {
    private SwitchLoopExceptionBoundary() {}

    private static int selector(int value) {
        if (value == 99) {
            throw new IllegalStateException("selector failure");
        }
        return value;
    }

    static int run(int[] values) {
        int total = 0;
        try {
            for (int i = 0; i < values.length; i++) {
                switch (selector(values[i])) {
                    case 3:
                        return total + i;
                    default:
                        total += selector(values[i]);
                }
            }
            return total;
        } catch (RuntimeException expected) {
            return -7;
        }
    }

    public static void main(String[] args) {
        System.out.println("empty=" + run(new int[0]));
        System.out.println("normal=" + run(new int[] {1, 2, 4}));
        System.out.println("case-return=" + run(new int[] {1, 3, 4}));
        System.out.println("caught=" + run(new int[] {1, 99, 4}));
        System.out.println("null=" + run(null));
    }
}

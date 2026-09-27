public final class HandlerLoopProbe {
    private static int effects;

    private static int work(int value) {
        if (value == 0) {
            throw new NumberFormatException("zero");
        }
        if (value == 2) {
            throw new IllegalStateException("two");
        }
        return value;
    }

    private static int run() {
        int total = 0;
        for (int value = 0; value < 3; value++) {
            try {
                try {
                    total += work(value);
                } catch (NumberFormatException ex) {
                    effects += 10;
                    continue;
                }
                total += 1;
            } catch (IllegalStateException ex) {
                effects += 100;
                total += 2;
            }
        }
        return total;
    }

    public static void main(String[] args) {
        System.out.println(run() + ":" + effects);
    }
}

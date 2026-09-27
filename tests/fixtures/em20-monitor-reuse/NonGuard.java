public class NonGuard {
    public static int catchThenLoop(int seed) {
        int limit;
        try {
            if (seed < 0) {
                throw new IllegalArgumentException();
            }
            limit = seed + 1;
        } catch (IllegalArgumentException ex) {
            limit = 0;
        }
        int sum = 0;
        for (int i = 0; i < limit; i++) {
            sum += i;
        }
        return sum;
    }

    public static int monitorInOuterLoop(int seed) {
        int sum = 0;
        for (int round = 0; round < 2; round++) {
            int limit;
            synchronized (NonGuard.class) {
                limit = seed + round;
            }
            for (int i = 0; i < limit; i++) {
                sum += i;
            }
        }
        return sum;
    }

    public static int laterHandlerReadsReusedInt(int seed) {
        int limit;
        synchronized (NonGuard.class) {
            limit = seed + 1;
        }
        int total = 0;
        try {
            total += limit;
            if (seed < 0) {
                throw new IllegalArgumentException();
            }
        } catch (IllegalArgumentException ex) {
            return total;
        }
        return total;
    }
}

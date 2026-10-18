public final class LoopArmExceptionBoundary {
    public static int run(int remaining, int seed) {
        int total = seed;
        while (remaining > 0) {
            try {
                total += remaining;
            } catch (RuntimeException failure) {
                return -1;
            }
            remaining--;
        }
        return total;
    }
}

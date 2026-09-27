package cf04;

public final class Negative {
    static int observed;

    public static int extraConsumer(boolean first, boolean second, boolean third) {
        return observed = (!first ? third : second) ? 1 : 2;
    }

    public static int protectedReturn(boolean first, boolean second, boolean third) {
        try {
            return (!choose(first) ? third : second) ? 1 : 2;
        } catch (RuntimeException ignored) {
            return 3;
        }
    }

    private static boolean choose(boolean first) {
        return first;
    }
}

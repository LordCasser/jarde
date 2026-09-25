public class BaseProbe {
    private static int sideEffectCount;
    private final int base;

    public BaseProbe(int base) {
        this.base = base;
    }

    public int value() {
        return base;
    }

    public int valueWith(int amount) {
        return base + amount;
    }

    public int failWith(int amount) {
        throw new IllegalStateException("base:" + amount);
    }

    public static int sideEffectArgument() {
        sideEffectCount++;
        return sideEffectCount;
    }

    public static int throwingArgument() {
        throw new IllegalArgumentException("argument");
    }

    public static int sideEffectCount() {
        return sideEffectCount;
    }
}

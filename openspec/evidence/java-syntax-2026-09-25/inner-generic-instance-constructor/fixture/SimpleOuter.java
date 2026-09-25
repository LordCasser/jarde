package nested;

public final class SimpleOuter {
    public static String trace = "";
    private final int bias;

    public SimpleOuter(int bias) {
        this.bias = bias;
    }

    public final class Inner {
        private final int value;

        public Inner(int value) {
            this.value = value;
        }

        public int value() {
            return bias + value;
        }
    }

    public static int mark(String label, int value) {
        trace += label;
        return value;
    }
}

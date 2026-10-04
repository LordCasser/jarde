abstract class Base {
    abstract void observe();
}

public final class ParameterizedAlsoConsumed {
    static String observed;

    // Negative (recover-anonymous-parameterized-root tasks 1.3, design Open Question (b)
    // resolved as default-refuse): the capture parameter is consumed a second time inside the
    // root method (the `echo` copy) beside the allocation argument. The value-flow proof
    // requires the parameter's entry value to reach exactly one use — the allocation argument —
    // so this shape keeps physical class text.
    private static Base create(final String captured) {
        final String echo = captured;
        return new Base() {
            @Override
            void observe() {
                observed = captured;
            }
        };
    }

    public static void main(String[] args) {
        create("captured-value");
        System.out.println("observed=" + observed);
    }
}

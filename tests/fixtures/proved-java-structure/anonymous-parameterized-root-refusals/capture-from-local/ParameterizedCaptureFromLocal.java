abstract class Base {
    abstract void observe();
}

public final class ParameterizedCaptureFromLocal {
    static String observed;

    // Negative (recover-anonymous-parameterized-root tasks 1.3): the root method's descriptor is
    // the single-parameter form "(Ljava/lang/String;)LBase;" but the allocation's capture
    // argument is a root local, not the parameter slot. The value-flow proof must refuse: the
    // parameterized gate accepts the descriptor only when the capture argument IS the unmodified
    // parameter slot 0.
    private static Base create(final String captured) {
        final String local = captured;
        return new Base() {
            @Override
            void observe() {
                observed = local;
            }
        };
    }

    public static void main(String[] args) {
        create("captured-value");
        System.out.println("observed=" + observed);
    }
}

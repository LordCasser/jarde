abstract class Base {
    abstract void observe();
}

public final class ParameterizedMultiParams {
    static String observed;

    // Negative (recover-anonymous-parameterized-root tasks 1.3): the root method takes two
    // parameters, so its descriptor can never equal the proved single capture parameter form
    // "(P)LBase;" nor the plain "()LBase;" form. `extra` is deliberately unused by the anonymous
    // body so the child still carries exactly one val$ capture field and the refusal lands on
    // the (relaxed) root return gate, not on the child shape.
    private static Base create(final String captured, final int extra) {
        return new Base() {
            @Override
            void observe() {
                observed = captured;
            }
        };
    }

    public static void main(String[] args) {
        create("captured-value", 7);
        System.out.println("observed=" + observed);
    }
}

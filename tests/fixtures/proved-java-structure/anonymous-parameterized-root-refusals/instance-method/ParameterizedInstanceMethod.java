abstract class Base {
    abstract void observe();
}

public final class ParameterizedInstanceMethod {
    static String observed;

    // Negative (recover-anonymous-parameterized-root tasks 1.3, design Open Question (a)
    // resolved as default-refuse): the root method is an instance method, so its slot 0 is the
    // receiver and the anonymous class carries the enclosing instance (`this$0`) beside the
    // capture field. The child shape gate refuses the two-field child; the projection's own
    // ACC_STATIC requirement stays as defense in depth for shapes javac cannot mint.
    private Base create(final String captured) {
        return new Base() {
            @Override
            void observe() {
                observed = captured;
            }
        };
    }

    public static void main(String[] args) {
        new ParameterizedInstanceMethod().create("captured-value");
        System.out.println("observed=" + observed);
    }
}

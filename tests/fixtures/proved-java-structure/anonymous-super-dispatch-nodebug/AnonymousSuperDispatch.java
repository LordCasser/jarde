abstract class Base {
    Base() {
        AnonymousSuperDispatch.inBaseConstructor = true;
        observe();
        AnonymousSuperDispatch.capturedVisibleBeforeBaseReturns =
                AnonymousSuperDispatch.inBaseConstructor
                        && "captured-value".equals(AnonymousSuperDispatch.observed);
        AnonymousSuperDispatch.inBaseConstructor = false;
    }

    abstract void observe();
}

public final class AnonymousSuperDispatch {
    static boolean inBaseConstructor;
    static boolean capturedVisibleBeforeBaseReturns;
    static String observed;

    private static Base create(final String captured) {
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
        System.out.println("visibleDuringSuper=" + capturedVisibleBeforeBaseReturns);
    }
}

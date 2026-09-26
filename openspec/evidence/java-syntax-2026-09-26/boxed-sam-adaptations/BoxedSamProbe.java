import java.util.function.Function;
import java.util.function.Supplier;

/** Java 8 bootstrap-adaptation fixture with observable boundary behavior. */
public final class BoxedSamProbe {
    private int supply() { return 7; }
    private int supplyOther() { return 9; }
    private int supplyMinimum() { return Integer.MIN_VALUE; }

    private static int staticRef(String value) { return value.length(); }
    private static int staticRefOther(String value) { return value.length() + 10; }

    Supplier<Integer> supplier() { return this::supply; }
    Supplier<Integer> minimumSupplier() { return this::supplyMinimum; }
    Function<String, Integer> function() { return BoxedSamProbe::staticRef; }
    Function<Integer, int[]> arrayCtor() { return int[]::new; }

    private int dispatch(Supplier<Integer> supplier, Function<String, Integer> function) {
        return supplier.get().intValue() + function.apply("x").intValue();
    }

    int chainedSites() {
        return dispatch(this::supply, BoxedSamProbe::staticRef)
                + dispatch(this::supplyOther, BoxedSamProbe::staticRefOther);
    }

    public static void main(String[] args) {
        BoxedSamProbe probe = new BoxedSamProbe();
        System.out.println("supplier=" + probe.supplier().get());
        System.out.println("minimum=" + probe.minimumSupplier().get());
        System.out.println("function=" + probe.function().apply("abc"));
        System.out.println("chained=" + probe.chainedSites());
        Function<Integer, int[]> arrays = probe.arrayCtor();
        System.out.println("array=" + arrays.apply(Integer.valueOf(3)).length);
        try {
            arrays.apply(null);
            System.out.println("null-array=returned");
        } catch (Throwable failure) {
            System.out.println("null-array=" + failure.getClass().getSimpleName());
        }
        try {
            arrays.apply(Integer.valueOf(-1));
            System.out.println("negative-array=returned");
        } catch (Throwable failure) {
            System.out.println("negative-array=" + failure.getClass().getSimpleName());
        }
    }
}

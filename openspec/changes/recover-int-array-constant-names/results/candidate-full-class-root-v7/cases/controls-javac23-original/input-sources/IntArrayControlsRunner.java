import java.util.Arrays;

public class IntArrayControlsRunner {
    public static void main(String[] args) {
        int[] uniqueFirst = UniqueIntArray.values();
        int[] uniqueSecond = UniqueIntArray.values();
        System.out.println("unique=" + Arrays.toString(uniqueFirst));
        System.out.println("unique-fresh=" + (uniqueFirst != uniqueSecond));

        System.out.println("duplicate=" + Arrays.toString(DuplicateIntArray.values()));
        System.out.println("shadow-parameter=" + Arrays.toString(ShadowIntArray.parameter(9)));
        System.out.println("shadow-local=" + Arrays.toString(ShadowIntArray.local()));
        System.out.println("unsupported-nested=" + Arrays.deepToString(UnsupportedIntArray.nested()));
        System.out.println("unsupported-leaves=" + Arrays.toString(UnsupportedIntArray.unsupportedLeaves(5L, 2)));
        System.out.println("prior-assert-ok=" + Arrays.toString(PriorAssertIntArray.value(true)));

        try {
            PriorAssertIntArray.value(false);
            System.out.println("prior-assert-false=not-thrown");
        } catch (AssertionError expected) {
            System.out.println("prior-assert-false=caught");
        }
    }
}

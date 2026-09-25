/** Source-only assertions for array, index, value order and store exceptions. */
public final class NarrowArrayStoreOrderRunner {
    private NarrowArrayStoreOrderRunner() {}

    private static String run(String label, int[] array, int index,
            boolean failArray, boolean failIndex, boolean failValue) {
        NarrowArrayStoreOrder.reset();
        try {
            NarrowArrayStoreOrder.store(array, index, 9,
                    failArray, failIndex, failValue);
            return label + ":ok:" + NarrowArrayStoreOrder.result()
                    + ":stored:" + (array == null ? "null" : array[0]);
        } catch (Throwable error) {
            return label + ":" + error.getClass().getName() + ":"
                    + NarrowArrayStoreOrder.result() + ":stored:"
                    + (array == null ? "null" : array[0]);
        }
    }

    private static void expect(String expected, String actual) {
        if (!expected.equals(actual)) {
            throw new AssertionError("expected [" + expected + "] got [" + actual + "]");
        }
        System.out.println(actual);
    }

    public static void main(String[] args) {
        int[] array = { 41 };
        expect("success:ok:AIV:1,1,1:stored:9", run("success", array, 0, false, false, false));
        array[0] = 41;
        expect("array-fail:java.lang.IllegalArgumentException:A:1,0,0:stored:41",
                run("array-fail", array, 0, true, false, false));
        expect("index-fail:java.lang.IllegalArgumentException:AI:1,1,0:stored:41",
                run("index-fail", array, 0, false, true, false));
        expect("value-fail:java.lang.IllegalStateException:AIV:1,1,1:stored:41",
                run("value-fail", array, 0, false, false, true));
        expect("null-value-fail:java.lang.IllegalStateException:AIV:1,1,1:stored:null",
                run("null-value-fail", null, 0, false, false, true));
        expect("null-store:java.lang.NullPointerException:AIV:1,1,1:stored:null",
                run("null-store", null, 0, false, false, false));
        expect("oob-value-fail:java.lang.IllegalStateException:AIV:1,1,1:stored:41",
                run("oob-value-fail", array, 1, false, false, true));
        expect("oob-store:java.lang.ArrayIndexOutOfBoundsException:AIV:1,1,1:stored:41",
                run("oob-store", array, 1, false, false, false));
    }
}

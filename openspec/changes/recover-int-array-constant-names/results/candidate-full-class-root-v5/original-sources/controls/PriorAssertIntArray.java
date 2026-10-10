public class PriorAssertIntArray {
    static final int VALUE = 7;

    static int[] value(boolean ok) {
        assert ok;
        return new int[] { 7 };
    }
}

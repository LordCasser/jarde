public class ShadowIntArray {
    static final int VALUE = 7;

    static int[] parameter(int VALUE) {
        return new int[] { 7, VALUE };
    }

    static int[] local() {
        int VALUE = 8;
        return new int[] { 7, VALUE };
    }
}

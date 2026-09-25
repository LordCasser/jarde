public final class JadxOrderingControl {
    static int trace;

    static int mark(int n) {
        trace = trace * 10 + n;
        return n;
    }

    static int[] build() {
        int[] result = new int[2];
        result[1] = mark(1);
        result[0] = mark(2);
        return result;
    }
}

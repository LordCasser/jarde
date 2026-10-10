public class CommonNoClinitArrayInit extends ArrayFieldInitBase {
    static int trace;
    final byte[] first;
    final byte[] second;

    static byte mark(int value) {
        trace = trace * 31 + value;
        return (byte) value;
    }

    static byte run(int value) {
        return mark(value);
    }

    public CommonNoClinitArrayInit() {
        super(7);
        first = new byte[] { mark(11), mark(12) };
        second = new byte[] { run(21) };
        trace = trace * 31 + 91;
    }

    public CommonNoClinitArrayInit(int marker) {
        super(marker);
        first = new byte[] { mark(11), mark(12) };
        second = new byte[] { run(21) };
        trace = trace * 31 + marker;
    }
}

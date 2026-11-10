public final class CaptureUpcastsDriver {
    public static void main(String[] args) {
        for (int input : new int[] { -17, 0, 23 }) {
            if (!"number".equals(CaptureUpcasts.one(input))) {
                throw new AssertionError("one(" + input + ")");
            }
        }
        if (!"object".equals(CaptureUpcasts.two())) {
            throw new AssertionError("two()");
        }
        System.out.println("PASS: one={number:-17,0,23}; two=object");
    }
}

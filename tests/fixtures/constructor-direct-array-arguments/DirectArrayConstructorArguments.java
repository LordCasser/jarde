public final class DirectArrayConstructorArguments {
    private static int trace;

    private DirectArrayConstructorArguments() {}

    private static int mark(int digit) {
        trace = trace * 10 + digit;
        return digit;
    }

    private static byte markByte(int digit) {
        trace = trace * 10 + digit;
        return (byte) digit;
    }

    public static final class ByteSink {
        final byte[] value;
        ByteSink(byte[] value) { this.value = value; }
    }

    public static final class IntSink {
        final int[] value;
        IntSink(int[] value) { this.value = value; }
    }

    public static final class ReferenceSink {
        final Object[] value;
        ReferenceSink(Object[] value) { this.value = value; }
    }

    public static final class MatrixSink {
        final int[][] value;
        MatrixSink(int[][] value) { this.value = value; }
    }

    public static final class ElementSink {
        final Element[] value;
        ElementSink(Element[] value) { this.value = value; }
    }

    public static final class PairSink {
        final byte[] bytes;
        final Object[] objects;
        PairSink(byte[] bytes, Object[] objects) {
            this.bytes = bytes;
            this.objects = objects;
        }
    }

    public static final class CharSink {
        final char[] value;
        CharSink(char[] value) { this.value = value; }
    }

    public static final class Element {
        final int value;
        Element(int value) { this.value = value; }
    }

    // Bare primitive and reference allocations. The length is dynamic so the proof cannot rely
    // on a compile-time zero or fixed-size special case.
    public static ByteSink barePrimitive(int length) {
        return new ByteSink(new byte[length]);
    }

    public static ReferenceSink bareReference(int length) {
        return new ReferenceSink(new Object[length]);
    }

    // Populated primitive and reference arrays; calls make element order observable.
    public static IntSink initializedPrimitive() {
        return new IntSink(new int[] {mark(1), mark(2)});
    }

    public static ReferenceSink initializedReference() {
        return new ReferenceSink(new Object[] {"left", "right"});
    }

    // One-dimensional child arrays assembled into an initialized parent.
    public static MatrixSink nestedChildren() {
        return new MatrixSink(new int[][] {new int[] {1, 2}, new int[] {3}});
    }

    // A bare multi-dimensional allocation (normally emitted as multianewarray).
    public static MatrixSink bareMultianewarray(int rows, int columns) {
        return new MatrixSink(new int[rows][columns]);
    }

    // Each reference-array element is itself a proved constructor expression.
    public static ElementSink constructedElements() {
        return new ElementSink(new Element[] {new Element(mark(3)), new Element(mark(4))});
    }

    // Both direct array arguments contain effects; their trace records left-to-right evaluation.
    public static PairSink twoArrayArguments() {
        trace = 0;
        PairSink result = new PairSink(
            new byte[] {markByte(5)},
            new Object[] {new Element(mark(6))}
        );
        if (trace != 56) throw new AssertionError("array argument order: " + trace);
        return result;
    }

    // Preserve the exact pre-existing Java 8 String(char[]) slice as a comparison case.
    public static String directStringCharArray() {
        return new String(new char[] {'a', 'b', 'c'});
    }

    // The exact same array fact under a different String constructor descriptor. This overlaps
    // the old String-specific wrongDescriptor negative and should be diagnosed separately because
    // the general complete-array proof may now be enough for the direct first argument.
    public static String stringOtherDescriptor() {
        return new String(new char[] {'d', 'e', 'f'}, 0, 3);
    }

    // The exact String(char[]) slice still has to reject an element whose effect is not part of its
    // constant-element certificate; do not let a generalized owner path bypass that branch.
    public static String directStringCharArrayEffectful() {
        return new String(new char[] {'a', (char) mark(7), 'c'});
    }

    // The same Java array expression under another owner exercises the general direct-argument
    // path. The old String-specific wrong-owner refusal may become a positive under that path.
    public static CharSink directOtherCharArrayOwner() {
        return new CharSink(new char[] {'x', 'y'});
    }
}

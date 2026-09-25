public final class BooleanOperandsRunner {
    private BooleanOperandsRunner() {}

    private static int numeric(Object value) {
        if (value instanceof Boolean) {
            return ((Boolean) value) ? 1 : 0;
        }
        if (value instanceof Character) {
            return (Character) value;
        }
        return ((Number) value).intValue();
    }

    public static void main(String[] args) {
        // Boxing keeps one runner source valid for Z and the patched B/C/S descriptors.
        System.out.println("run(false)=" + numeric(BooleanOperands.run(false)));
        System.out.println("run(true)=" + numeric(BooleanOperands.run(true)));
    }
}

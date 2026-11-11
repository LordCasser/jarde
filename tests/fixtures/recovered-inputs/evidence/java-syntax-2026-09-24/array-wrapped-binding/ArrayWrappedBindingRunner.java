final class ArrayWrappedBindingRunner {
    private static void run(String label, int which, int[] a) {
        ArrayWrappedBinding.calls = 0;
        ArrayWrappedBinding.failure = label.startsWith("throw-")
                ? new IllegalStateException("boom") : null;
        try {
            int value;
            switch (which) {
                case 0: value = ArrayWrappedBinding.plusArrayFirst(a); break;
                case 1: value = ArrayWrappedBinding.plusTickFirst(a); break;
                case 2: value = ArrayWrappedBinding.callTickThenArray(a); break;
                case 3: value = ArrayWrappedBinding.callMutateThenArray(a); break;
                case 4: value = ArrayWrappedBinding.readAfterEffect(a); break;
                case 5: value = ArrayWrappedBinding.twoReadsWithMutation(a); break;
                default: value = ArrayWrappedBinding.wrappedOnlyRead(a); break;
            }
            System.out.println(label + "=" + value + ",calls=" + ArrayWrappedBinding.calls
                    + ",array=" + (a == null ? "null" : java.util.Arrays.toString(a)));
        } catch (Throwable t) {
            System.out.println(label + "=" + t.getClass().getSimpleName() + ":" + t.getMessage()
                    + ",calls=" + ArrayWrappedBinding.calls
                    + ",array=" + (a == null ? "null" : java.util.Arrays.toString(a)));
        }
    }

    public static void main(String[] args) {
        run("array-first", 0, new int[] {3});
        run("tick-first", 1, new int[] {3});
        run("call-tick-array", 2, new int[] {3});
        run("call-mutate-array", 3, new int[] {3});
        run("read-after-effect", 4, new int[] {3});
        run("two-reads", 5, new int[] {3});
        run("wrapped-only", 6, new int[] {3});
        run("throw-array-first", 0, new int[] {3});
        run("throw-tick-first", 1, new int[] {3});
        run("throw-call", 2, new int[] {3});
        run("null-plus-array", 0, null);
        run("null-tick-first", 1, null);
        run("null-call", 2, null);
    }
}

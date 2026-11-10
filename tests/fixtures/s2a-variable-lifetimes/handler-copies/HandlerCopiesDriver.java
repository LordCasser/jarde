public final class HandlerCopiesDriver {
    public static void main(String[] args) {
        int[] inputs = {Integer.MIN_VALUE, -1, 0, 7};
        for (int input : inputs) {
            check("copy false", HandlerCopies.copy(false, input), input);
            check("copy true", HandlerCopies.copy(true, input), input + input);
            check("initialized false", HandlerCopies.initializedByCall(false, input), 6);
            check("initialized true", HandlerCopies.initializedByCall(true, input), 6 + input);
        }
        System.out.println("PASS: copy and initializedByCall; fail={false,true}; input={MIN,-1,0,7}");
    }

    private static void check(String label, int actual, int expected) {
        if (actual != expected) throw new AssertionError(label + " expected=" + expected + " actual=" + actual);
    }
}

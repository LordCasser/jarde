public final class Runner {
    private static void run(String label, int a, int fail) {
        ConditionalIntermediateJoin.reset(fail);
        try {
            int value = ConditionalIntermediateJoin.choose(a);
            System.out.println(label + "=value:" + value + ":" + ConditionalIntermediateJoin.trace());
        } catch (RuntimeException ex) {
            System.out.println(label + "=throw:" + ex.getClass().getSimpleName() + ":" + ex.getMessage()
                    + ":" + ConditionalIntermediateJoin.trace());
        }
    }

    public static void main(String[] args) {
        run("outer-false", 0, 0);
        run("inner-first", 2, 0);
        run("inner-second", 1, 0);
        run("inner-first-throws", 2, 1);
        run("inner-second-throws", 1, 2);
        run("outer-throws", 0, 3);
    }
}

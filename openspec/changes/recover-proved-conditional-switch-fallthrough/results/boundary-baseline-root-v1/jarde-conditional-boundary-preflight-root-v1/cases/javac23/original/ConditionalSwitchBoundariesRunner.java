public final class ConditionalSwitchBoundariesRunner {
    private ConditionalSwitchBoundariesRunner() {}

    private static String terminal(int selector) {
        try {
            return "return:" + ConditionalSwitchBoundaries.terminalCase(selector);
        } catch (IllegalArgumentException exception) {
            return "throw:" + exception.getClass().getSimpleName() + ":" + exception.getMessage();
        }
    }

    public static void main(String[] args) {
        int[] selectors = {-1, 0, 1, 2};
        for (int selector : selectors) {
            for (int flag = 0; flag <= 1; flag++) {
                System.out.println("partial|s=" + selector + "|f=" + flag + "|"
                        + ConditionalSwitchBoundaries.partialBreak(selector, flag));
                System.out.println("loop|s=" + selector + "|f=" + flag + "|"
                        + ConditionalSwitchBoundaries.innerLoopBreak(selector, flag));
                System.out.println("caught|s=" + selector + "|f=" + flag + "|"
                        + ConditionalSwitchBoundaries.caughtExceptionThenFallthrough(selector, flag));
            }
            for (int inner = 0; inner <= 1; inner++) {
                System.out.println("inner-switch|s=" + selector + "|i=" + inner + "|"
                        + ConditionalSwitchBoundaries.innerSwitchBreak(selector, inner));
            }
            System.out.println("terminal|s=" + selector + "|" + terminal(selector));
        }
    }
}

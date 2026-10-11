public final class Runner {
    private static void run(String label, int nextMode, int target, int lower) {
        ComputedForEffectControl.reset(nextMode);
        int result = 777;
        String outcome = "normal";
        try {
            result = ComputedForEffectControl.find(
                new int[] {10, 20, 30}, target, lower, 0, 0, 0);
        } catch (RuntimeException failure) {
            if ("fail-f".equals(failure.getMessage())) {
                outcome = "fail-f";
            } else if ("fail-g".equals(failure.getMessage())) {
                outcome = "fail-g";
            } else {
                outcome = "other-exception";
            }
        }
        System.out.println(label + " outcome=" + outcome + " result=" + result
            + " f=" + ComputedForEffectControl.fCalls
            + " g=" + ComputedForEffectControl.gCalls
            + " order=" + ComputedForEffectControl.order
            + " end=" + ComputedForEffectControl.end);
    }

    public static void main(String[] args) {
        run("normal-hit", 0, 20, 0);
        run("normal-miss", 0, 99, 0);
        run("normal-empty-range", 0, 20, 3);
        run("fail-f", 1, 20, 0);
        run("fail-g", 2, 20, 0);
    }
}

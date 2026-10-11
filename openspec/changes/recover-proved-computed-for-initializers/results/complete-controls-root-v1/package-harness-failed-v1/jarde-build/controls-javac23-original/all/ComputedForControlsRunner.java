public final class ComputedForControlsRunner {
    private static void reset(int selectedMode) {
        ComputedForControls.end = 6;
        ComputedForControls.trace = 0;
        ComputedForControls.mode = selectedMode;
    }

    private static void show(String label, int selectedMode, int method, int target) {
        reset(selectedMode);
        int result;
        try {
            switch (method) {
                case 0: result = ComputedForControls.find(new int[] {2, 4, 6, 8, 10, 12}, target, 0); break;
                case 1: result = ComputedForControls.shared(0); break;
                case 2: result = ComputedForControls.observedAfter(0); break;
                case 3: result = ComputedForControls.externalWrite(0); break;
                case 4: result = ComputedForControls.scoped(0, target != 0); break;
                default: throw new AssertionError(method);
            }
            System.out.println(label + ":" + result + ":" + ComputedForControls.trace + ":" + ComputedForControls.end);
        } catch (RuntimeException failure) {
            System.out.println(label + ":throw:" + (failure == ComputedForControls.FAILURE) + ":" + ComputedForControls.trace);
        }
    }

    public static void main(String[] args) {
        show("hit-after-g-mutates-end", 0, 0, 6);
        show("miss", 0, 0, 99);
        show("f-throws-before-g", 1, 0, 6);
        show("g-throws-after-f", 2, 0, 6);
        show("shared-f-value", 0, 1, 0);
        show("post-loop-read", 0, 2, 0);
        show("post-loop-write", 0, 3, 0);
        show("nested-scope-enter", 0, 4, 1);
        show("nested-scope-skip", 0, 4, 0);
    }
}

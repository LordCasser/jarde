public class VerifySharedTrueControls {
    private static void show(String className, boolean result, int calls) {
        System.out.println(className + ":" + result + "," + calls);
    }

    public static void main(String[] args) {
        SharedTrueDuplicatePhi.assign(true);
        show("duplicate-true", SharedTrueDuplicatePhi.result, SharedTrueDuplicatePhi.calls);
        SharedTrueDuplicatePhi.calls = 0;
        SharedTrueDuplicatePhi.assign(false);
        show("duplicate-false", SharedTrueDuplicatePhi.result, SharedTrueDuplicatePhi.calls);

        SharedTrueShortCircuit.assign(true);
        show("single-true", SharedTrueShortCircuit.result, SharedTrueShortCircuit.calls);
        SharedTrueShortCircuit.calls = 0;
        SharedTrueShortCircuit.assign(false);
        show("single-false", SharedTrueShortCircuit.result, SharedTrueShortCircuit.calls);
    }
}

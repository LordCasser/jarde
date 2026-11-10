public final class DevDriver {
    public static void main(String[] args) {
        check("merge-left", DevCases.mergeSlots(true, 10), 54);
        check("merge-right", DevCases.mergeSlots(false, 10), 3);
        check("sequential-slots", DevCases.sequentialSlotLifetimes(3), 16);
        check("branch-local-left", DevCases.branchLocalReferences(true, 2), "left-2");
        check("branch-local-right", DevCases.branchLocalReferences(false, 2), "right-2");
        check("swap-0", DevCases.swapAcrossBackedge(2, 5, 0), 19);
        check("swap-3", DevCases.swapAcrossBackedge(2, 5, 3), 418);
        check("critical-skip-peer", DevCases.criticalThenSibling(false, true, 8), 16);
        check("critical-update-peer", DevCases.criticalThenSibling(true, true, 8), 26);
        check("critical-skip-return", DevCases.criticalThenSibling(false, false, 8), 5);
        check("handler-return", DevCases.handlerSeesPreThrowLocal(false, 4), 6);
        check("handler-throw", DevCases.handlerSeesPreThrowLocal(true, 4), 187);
        System.out.println("PASS dev 12 checks");
    }
    private static void check(String name, String actual, String expected) {
        if (!expected.equals(actual)) throw new AssertionError(name + ": expected=" + expected + " actual=" + actual);
        System.out.println("OK " + name + "=" + actual);
    }
    private static void check(String name, int actual, int expected) {
        if (actual != expected) throw new AssertionError(name + ": expected=" + expected + " actual=" + actual);
        System.out.println("OK " + name + "=" + actual);
    }
}

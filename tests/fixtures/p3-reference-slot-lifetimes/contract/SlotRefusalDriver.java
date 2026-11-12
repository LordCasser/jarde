public final class SlotRefusalDriver {
    private static void check(String label, long actual, long expected) {
        if (actual != expected) throw new AssertionError(label + ": " + actual + " != " + expected);
        System.out.println(label + "=" + actual);
    }
    public static void main(String[] args) {
        check("loop(-2)", SlotReuseBoundaries.loopBodyReuse(-2), 0);
        check("loop(-1)", SlotReuseBoundaries.loopBodyReuse(-1), 0);
        check("loop(0)", SlotReuseBoundaries.loopBodyReuse(0), 0);
        check("loop(1)", SlotReuseBoundaries.loopBodyReuse(1), 3);
        check("loop(2)", SlotReuseBoundaries.loopBodyReuse(2), 8);
        check("loop(3)", SlotReuseBoundaries.loopBodyReuse(3), 15);
        check("sameType(-2)", SlotReuseBoundaries.sameType(-2), -1);
        check("sameType(0)", SlotReuseBoundaries.sameType(0), 3);
        check("sameType(4)", SlotReuseBoundaries.sameType(4), 11);
        check("exclusive(false)", SlotReuseBoundaries.exclusiveBranch(false), 9);
        check("exclusive(true)", SlotReuseBoundaries.exclusiveBranch(true), 7);
        check("handler(false)", SlotReuseBoundaries.handlerReuse(false), 3);
        check("handler(true)", SlotReuseBoundaries.handlerReuse(true), 8);
        check("category2", SlotReuseBoundaries.category2Adjacent(), 4294967300L);
    }
}

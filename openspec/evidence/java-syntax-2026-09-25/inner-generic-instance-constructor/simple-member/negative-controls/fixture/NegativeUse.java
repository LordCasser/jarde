package negative;

public final class NegativeUse {
    public static Object nestedEffects(NegativeOuter outer, int value) {
        return outer.new Inner(NegativeOuter.mark("A",
                NegativeOuter.mark("B", value)));
    }

    public static Object preEffect(NegativeOuter outer, int value) {
        int prepared = NegativeOuter.mark("P", value);
        return outer.new Inner(NegativeOuter.mark("A", prepared));
    }
}

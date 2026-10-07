// The `Target` of the hand-built `SPC.class`: its class initializer runs when the allocation is
// executed, which is the order the statement position must keep.
public final class Target {
    static {
        Trace.value += "C";
    }

    public Target(int value) {
        Trace.value += "T";
    }
}

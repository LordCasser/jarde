// jarde: presentation of `BoundaryProbe` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public interface BoundaryProbe {
    public static final java.lang.String FIRST;

    public static final java.lang.String SECOND;

    public static java.lang.String observe() {
        // @method observe()Ljava/lang/String;
        // @declaration an interface's static method of `BoundaryProbe`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return new java.lang.StringBuilder().append(BoundaryEffects.trace).append("|").append(BoundaryProbe.FIRST).append("|").append(BoundaryProbe.SECOND).toString();
    }

    public static void patchAnchor() {
        // @method patchAnchor()V
        // @declaration an interface's static method of `BoundaryProbe`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        BoundaryEffects.independent();
        return;
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `BoundaryProbe`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        if (BoundaryEffects.choose()) {
            // @bytecode 8
            // the saved producer at BCI 8 has no bounded final expression consumer
        } else {
            // @bytecode 16
            // the saved producer at BCI 16 has no bounded final expression consumer
        }
        // @bytecode 19
        // the value at BCI 19 is the entry state of stack depth 0, which no instruction produced
        BoundaryProbe.SECOND = BoundaryEffects.next("B");
    }
}

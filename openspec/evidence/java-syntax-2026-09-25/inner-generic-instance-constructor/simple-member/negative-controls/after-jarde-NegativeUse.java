// jarde: presentation of `negative/NegativeUse` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package negative;

public final class NegativeUse extends java.lang.Object {
    public NegativeUse() {
        // @method <init>()V
        // @declaration a constructor of `negative.NegativeUse`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static java.lang.Object nestedEffects(negative.NegativeOuter arg0, int arg1) {
        // @method nestedEffects(Lnegative/NegativeOuter;I)Ljava/lang/Object;
        // @declaration a static method of `negative.NegativeUse`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return arg0.new Inner(negative.NegativeOuter.mark("A", negative.NegativeOuter.mark("B", arg1)));
    }

    public static java.lang.Object preEffect(negative.NegativeOuter arg0, int arg1) {
        // @method preEffect(Lnegative/NegativeOuter;I)Ljava/lang/Object;
        // @declaration a static method of `negative.NegativeUse`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local2 = negative.NegativeOuter.mark("P", arg1);
        return arg0.new Inner(negative.NegativeOuter.mark("A", local2));
    }
}

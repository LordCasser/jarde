// jarde: presentation of `BooleanOperands` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class BooleanOperands extends java.lang.Object {
    private BooleanOperands() {
        // @method <init>()V
        // @declaration a constructor of `BooleanOperands`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static short run(boolean arg0) {
        // jarde: not recovered: the recovery run for `run(Z)S` produced no statement (explanation only); the artifact's own comment lines are below
        // @method run(Z)S
        // @declaration a static method of `BooleanOperands`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 1
        // the value at BCI 0 is presented as `boolean` and the member's own descriptor returns `short`, and no conversion this layer's evidence states connects the two: a widening primitive conversion (JLS 5.1.2) is the only one a position performs for itself, so the region is refused rather than published with the value the position would convert differently or with text `javac` refuses
    }
}

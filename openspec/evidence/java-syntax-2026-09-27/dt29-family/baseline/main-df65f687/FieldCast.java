// jarde: presentation of `dt29/FieldCast` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package dt29;

public class FieldCast extends java.lang.Object {
    public FieldCast() {
        // @method <init>()V
        // @declaration a constructor of `dt29.FieldCast`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static java.lang.String run() {
        // @method run()Ljava/lang/String;
        // @declaration a static method of `dt29.FieldCast`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        dt29.FieldCast$B local0 = new dt29.FieldCast$B();
        local0.self(true);
        // @bytecode 17 14 13
        // the parameter 0 of the invocation at BCI 14 is declared `dt29.FieldCast$A` presents `dt29.FieldCast$B` but the invocation requires `dt29.FieldCast$A` and this layer has no safe reference conversion evidence
        new dt29.FieldCast$C().set(local0, false);
        // @bytecode 34 31 30
        // the parameter 0 of the invocation at BCI 31 is declared `dt29.FieldCast$A` presents `dt29.FieldCast$B` but the invocation requires `dt29.FieldCast$A` and this layer has no safe reference conversion evidence
        new dt29.FieldCast$D((dt29.FieldCast$1) null).set(local0, true);
        // @bytecode 83
        // the parameter 0 of the invocation at BCI 74 is declared `dt29.FieldCast$A` presents `dt29.FieldCast$B` but the invocation requires `dt29.FieldCast$A` and this layer has no safe reference conversion evidence
    }

    private static java.lang.String bits(dt29.FieldCast$A arg0) {
        // jarde: not recovered: the recovery run for `bits(Ldt29/FieldCast$A;)Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method bits(Ldt29/FieldCast$A;)Ljava/lang/String;
        // @declaration a static method of `dt29.FieldCast`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 3 4 7 8 11 14 16 19
        // the carried argument crosses an independent instruction at BCI 21
        // @bytecode 21 24 25 28 31 33 36
        // the carried argument crosses an independent instruction at BCI 21
        // @bytecode 38 41 42 45 48 50 53
        // the carried argument crosses an independent instruction at BCI 55
        // @bytecode 55 58 59 62 65 67 70
        // the carried argument crosses an independent instruction at BCI 55
        // @bytecode 72
        // the dependency chain from BCI 72 to final consumer 78 is not bounded
        // @bytecode 72 75
        // the dependency chain from BCI 75 to final consumer 78 is not bounded
        // @bytecode 78 75 72
        // the value at BCI 78 was produced by a saved declaration this run could not commit
    }
}

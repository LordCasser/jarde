// jarde: presentation of `dt29/FieldCast$B` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package dt29;

public class FieldCast$B extends dt29.FieldCast$A {
    public FieldCast$B() {
        // @method <init>()V
        // @declaration a constructor of `dt29.FieldCast$B`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public void self(boolean arg1) {
        // @method self(Z)V
        // @declaration an instance method of `dt29.FieldCast$B`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 2
        // the field access at BCI 2 is not one this run proved names the member its own receiver's type declares, and a field instruction is presented only where the member it names is proven
        // @bytecode 7
        // the field access at BCI 7 is not one this run proved names the member its own receiver's type declares, and a field instruction is presented only where the member it names is proven
        // @bytecode 12
        // the field access at BCI 12 is not one this run proved names the member its own receiver's type declares, and a field instruction is presented only where the member it names is proven
        // @bytecode 17 20 15 16
        // the parameter 0 of the invocation at BCI 17 is declared `dt29.FieldCast$A` presents `dt29.FieldCast$B` but the invocation requires `dt29.FieldCast$A` and this layer has no safe reference conversion evidence
        return;
    }
}

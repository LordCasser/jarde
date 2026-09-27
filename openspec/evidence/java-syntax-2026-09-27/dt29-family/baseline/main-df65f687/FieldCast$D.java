// jarde: presentation of `dt29/FieldCast$D` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package dt29;

class FieldCast$D extends java.lang.Object {
    private FieldCast$D() {
        // @method <init>()V
        // @declaration a constructor of `dt29.FieldCast$D`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    // jarde: generic Signature projection refused for `set(Ldt29/FieldCast$B;Z)V`: unsupported (generic_source_shape_unproved): the recovered AST/SSA body is not a direct parameter return
    public void set(dt29.FieldCast$B arg1, boolean arg2) {
        // @method set(Ldt29/FieldCast$B;Z)V
        // @declaration an instance method of `dt29.FieldCast$D`, member flags 0x0001
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

    FieldCast$D(dt29.FieldCast$1 arg1) {
        // @method <init>(Ldt29/FieldCast$1;)V
        // @declaration a constructor of `dt29.FieldCast$D`, member flags 0x1000
        // recovered from bytecode; presentation is not claimed to compile
        this();
        return;
    }
}

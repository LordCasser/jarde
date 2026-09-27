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

    public <T extends dt29.FieldCast$B> void set(T arg1, boolean arg2) {
        // jarde: generic Signature `<T:Ldt29/FieldCast$B;>(TT;Z)V` projected after descriptor erasure and same-run complete straight-line AST/Code/SSA parameter-use and Signature erasure proof
        // @method set(Ldt29/FieldCast$B;Z)V
        // @declaration an instance method of `dt29.FieldCast$D`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        ((dt29.FieldCast$A) arg1).publicField = arg2;
        ((dt29.FieldCast$A) arg1).protectedField = arg2;
        ((dt29.FieldCast$A) arg1).packagePrivateField = arg2;
        dt29.FieldCast$A.access$002((dt29.FieldCast$A) arg1, arg2);
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

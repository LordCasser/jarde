// jarde: presentation of `dt29/FieldCast$A` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package dt29;

public class FieldCast$A extends java.lang.Object {
    public boolean publicField;

    boolean packagePrivateField;

    protected boolean protectedField;

    private boolean privateField;

    public FieldCast$A() {
        // @method <init>()V
        // @declaration a constructor of `dt29.FieldCast$A`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static boolean access$002(dt29.FieldCast$A arg0, boolean arg1) {
        // @method access$002(Ldt29/FieldCast$A;Z)Z
        // @declaration a static method of `dt29.FieldCast$A`, member flags 0x1008
        // recovered from bytecode; presentation is not claimed to compile
        arg0.privateField = arg1;
        return arg1;
    }

    static boolean access$000(dt29.FieldCast$A arg0) {
        // @method access$000(Ldt29/FieldCast$A;)Z
        // @declaration a static method of `dt29.FieldCast$A`, member flags 0x1008
        // recovered from bytecode; presentation is not claimed to compile
        return arg0.privateField;
    }
}

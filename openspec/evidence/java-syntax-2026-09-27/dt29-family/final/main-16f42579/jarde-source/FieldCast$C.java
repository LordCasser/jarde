// jarde: presentation of `dt29/FieldCast$C` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package dt29;

public class FieldCast$C extends java.lang.Object {
    public FieldCast$C() {
        // @method <init>()V
        // @declaration a constructor of `dt29.FieldCast$C`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public void set(dt29.FieldCast$B arg1, boolean arg2) {
        // @method set(Ldt29/FieldCast$B;Z)V
        // @declaration an instance method of `dt29.FieldCast$C`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        ((dt29.FieldCast$A) arg1).publicField = arg2;
        ((dt29.FieldCast$A) arg1).protectedField = arg2;
        ((dt29.FieldCast$A) arg1).packagePrivateField = arg2;
        dt29.FieldCast$A.access$002((dt29.FieldCast$A) arg1, arg2);
        return;
    }
}

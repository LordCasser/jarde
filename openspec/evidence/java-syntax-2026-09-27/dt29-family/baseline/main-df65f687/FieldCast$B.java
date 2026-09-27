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
        ((dt29.FieldCast$A) this).publicField = arg1;
        ((dt29.FieldCast$A) this).protectedField = arg1;
        ((dt29.FieldCast$A) this).packagePrivateField = arg1;
        dt29.FieldCast$A.access$002((dt29.FieldCast$A) this, arg1);
        return;
    }
}

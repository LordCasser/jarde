// jarde: presentation of `dt29/SamePackageParentFamily$B` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package dt29;

public class SamePackageParentFamily$B extends dt29.SamePackageParentFamily$A {
    protected boolean protectedField;

    boolean packagePrivateField;

    public SamePackageParentFamily$B() {
        // @method <init>()V
        // @declaration a constructor of `dt29.SamePackageParentFamily$B`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public void set(boolean arg1, boolean arg2) {
        // @method set(ZZ)V
        // @declaration an instance method of `dt29.SamePackageParentFamily$B`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        ((dt29.SamePackageParentFamily$A) this).publicField = arg1;
        ((dt29.SamePackageParentFamily$A) this).protectedField = arg1;
        ((dt29.SamePackageParentFamily$A) this).packagePrivateField = arg1;
        dt29.SamePackageParentFamily$A.access$002((dt29.SamePackageParentFamily$A) this, arg2);
        return;
    }
}

// jarde: presentation of `dt29/SamePackageParentFamily$A` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package dt29;

public class SamePackageParentFamily$A extends java.lang.Object {
    public boolean publicField;

    protected boolean protectedField;

    boolean packagePrivateField;

    private boolean privateField;

    public static boolean staticField;

    int descriptorControl;

    public SamePackageParentFamily$A() {
        // @method <init>()V
        // @declaration a constructor of `dt29.SamePackageParentFamily$A`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static boolean access$002(dt29.SamePackageParentFamily$A arg0, boolean arg1) {
        // @method access$002(Ldt29/SamePackageParentFamily$A;Z)Z
        // @declaration a static method of `dt29.SamePackageParentFamily$A`, member flags 0x1008
        // recovered from bytecode; presentation is not claimed to compile
        arg0.privateField = arg1;
        return arg1;
    }
}

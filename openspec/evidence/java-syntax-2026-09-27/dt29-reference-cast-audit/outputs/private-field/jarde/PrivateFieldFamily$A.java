// jarde: presentation of `dt29/PrivateFieldFamily$A` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package dt29;

public class PrivateFieldFamily$A extends java.lang.Object {
    public boolean visible;

    private boolean hidden;

    public PrivateFieldFamily$A() {
        // @method <init>()V
        // @declaration a constructor of `dt29.PrivateFieldFamily$A`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static boolean access$002(dt29.PrivateFieldFamily$A arg0, boolean arg1) {
        // @method access$002(Ldt29/PrivateFieldFamily$A;Z)Z
        // @declaration a static method of `dt29.PrivateFieldFamily$A`, member flags 0x1008
        // recovered from bytecode; presentation is not claimed to compile
        arg0.hidden = arg1;
        return arg1;
    }
}

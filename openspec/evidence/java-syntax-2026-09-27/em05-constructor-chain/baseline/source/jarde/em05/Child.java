// jarde: presentation of `em05/Child` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package em05;

public class Child extends em05.Base {
    public Child() {
        // @method <init>()V
        // @declaration a constructor of `em05.Child`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public Child(java.lang.String arg1) {
        // @method <init>(Ljava/lang/String;)V
        // @declaration a constructor of `em05.Child`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super(arg1);
        return;
    }

    public Child(int arg1) {
        // @method <init>(I)V
        // @declaration a constructor of `em05.Child`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        this((java.lang.String) java.lang.Integer.toString(arg1));
        return;
    }
}

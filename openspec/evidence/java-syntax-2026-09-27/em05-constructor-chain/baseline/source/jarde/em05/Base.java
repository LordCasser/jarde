// jarde: presentation of `em05/Base` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package em05;

public class Base extends java.lang.Object {
    public final java.lang.String value;

    public Base() {
        // @method <init>()V
        // @declaration a constructor of `em05.Base`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.value = "a";
        return;
    }

    public Base(java.lang.String arg1) {
        // @method <init>(Ljava/lang/String;)V
        // @declaration a constructor of `em05.Base`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.value = arg1;
        return;
    }
}

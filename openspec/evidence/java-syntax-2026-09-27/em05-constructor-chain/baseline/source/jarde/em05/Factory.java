// jarde: presentation of `em05/Factory` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package em05;

public class Factory extends java.lang.Object {
    public Factory() {
        // @method <init>()V
        // @declaration a constructor of `em05.Factory`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static em05.Child choose(boolean arg0) {
        // @method choose(Z)Lem05/Child;
        // @declaration a static method of `em05.Factory`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        em05.Child local1;
        if (arg0) {
            local1 = new em05.Child();
        } else {
            local1 = new em05.Child("b");
        }
        return local1;
    }

    public static em05.Child chain(java.lang.String arg0) {
        // @method chain(Ljava/lang/String;)Lem05/Child;
        // @declaration a static method of `em05.Factory`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return new em05.Child(arg0);
    }
}

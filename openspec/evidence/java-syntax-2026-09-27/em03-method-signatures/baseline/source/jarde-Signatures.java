// jarde: presentation of `em03/Signatures` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package em03;

public class Signatures extends java.lang.Object {
    public Signatures() {
        // @method <init>()V
        // @declaration a constructor of `em03.Signatures`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public java.lang.String named(java.lang.String arg1, int arg2) {
        // @method named(Ljava/lang/String;I)Ljava/lang/String;
        // @declaration an instance method of `em03.Signatures`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return arg1;
    }

    public int declared(int arg1) throws java.io.IOException {
        // @method declared(I)I
        // @declaration an instance method of `em03.Signatures`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return arg1;
    }

    public void raises() throws java.io.IOException {
        // @method raises()V
        // @declaration an instance method of `em03.Signatures`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        throw new java.io.IOException("negative");
    }
}

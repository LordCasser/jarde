// jarde: presentation of `ArrayFieldLiteral` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class ArrayFieldLiteral extends java.lang.Object {
    static byte[] a = new byte[]{10, 20, 30};

    byte[] b = new byte[]{40, 50, 60};

    public ArrayFieldLiteral() {
        // @method <init>()V
        // @declaration a constructor of `ArrayFieldLiteral`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }
}

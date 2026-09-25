// jarde: presentation of `ArrayObject` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class ArrayObject extends java.lang.Object {
    public ArrayObject() {
        // @method <init>()V
        // @declaration a constructor of `ArrayObject`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static int arr(java.lang.Object arg0) {
        // @method arr(Ljava/lang/Object;)I
        // @declaration a static method of `ArrayObject`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return 3;
    }

    public static int arr(java.lang.String[] arg0) {
        // @method arr([Ljava/lang/String;)I
        // @declaration a static method of `ArrayObject`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return 4;
    }

    public static int run(java.lang.String[] arg0) {
        // @method run([Ljava/lang/String;)I
        // @declaration a static method of `ArrayObject`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return arr(arg0);
    }
}

// jarde: presentation of `ArrayFieldInitializers` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class ArrayFieldInitializers extends java.lang.Object {
    static int trace = 0;

    static byte before = mark(1);

    static byte[] a = new byte[]{mark(2), mark(3), mark(4)};

    static byte after = mark(5);

    byte[] b;

    public ArrayFieldInitializers() {
        // @method <init>()V
        // @declaration a constructor of `ArrayFieldInitializers`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.b = new byte[]{mark(6), mark(7), mark(8)};
        mark(9);
        return;
    }

    static byte mark(int arg0) {
        // @method mark(I)B
        // @declaration a static method of `ArrayFieldInitializers`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        ArrayFieldInitializers.trace = ArrayFieldInitializers.trace * 10 + arg0;
        return (byte) arg0;
    }
}

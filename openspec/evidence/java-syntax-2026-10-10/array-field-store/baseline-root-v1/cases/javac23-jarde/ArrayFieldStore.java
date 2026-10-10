// jarde: presentation of `ArrayFieldStore` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class ArrayFieldStore extends java.lang.Object {
    private byte[] values;

    private static int trace;

    public ArrayFieldStore() {
        // @method <init>()V
        // @declaration a constructor of `ArrayFieldStore`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public void storeLiteral() {
        // @method storeLiteral()V
        // @declaration an instance method of `ArrayFieldStore`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        this.values = new byte[]{10, 20, 30};
        return;
    }

    public static void replace(ArrayFieldStore arg0, int arg1) {
        // @method replace(LArrayFieldStore;I)V
        // @declaration a static method of `ArrayFieldStore`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        arg0.values = new byte[]{element(1, (byte) 11, arg1), element(2, (byte) 22, arg1), element(3, (byte) 33, arg1)};
        return;
    }

    private static byte element(int arg0, byte arg1, int arg2) {
        // @method element(IBI)B
        // @declaration a static method of `ArrayFieldStore`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        ArrayFieldStore.trace = ArrayFieldStore.trace * 10 + arg0;
        if (arg0 == arg2) {
            throw new java.lang.IllegalStateException("element-" + arg0);
        } else {
            return arg1;
        }
    }

    public byte[] readValues() {
        // @method readValues()[B
        // @declaration an instance method of `ArrayFieldStore`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.values;
    }

    public static void resetTrace() {
        // @method resetTrace()V
        // @declaration a static method of `ArrayFieldStore`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        ArrayFieldStore.trace = 0;
        return;
    }

    public static int readTrace() {
        // @method readTrace()I
        // @declaration a static method of `ArrayFieldStore`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return ArrayFieldStore.trace;
    }
}

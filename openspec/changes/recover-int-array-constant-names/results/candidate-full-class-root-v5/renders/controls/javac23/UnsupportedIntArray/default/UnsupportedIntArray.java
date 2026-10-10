// jarde: presentation of `UnsupportedIntArray` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class UnsupportedIntArray extends java.lang.Object {
    static final int VALUE = 7;

    public UnsupportedIntArray() {
        // @method <init>()V
        // @declaration a constructor of `UnsupportedIntArray`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static int make() {
        // @method make()I
        // @declaration a static method of `UnsupportedIntArray`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return 7;
    }

    static int[][] nested() {
        // @method nested()[[I
        // @declaration a static method of `UnsupportedIntArray`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return new int[][]{new int[]{7}};
    }

    static int[] unsupportedLeaves(long source, int offset) {
        // @method unsupportedLeaves(JI)[I
        // @declaration a static method of `UnsupportedIntArray`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return new int[]{make(), (int) source, offset + 7};
    }
}

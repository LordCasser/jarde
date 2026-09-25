// jarde: presentation of `NestedArrayExtraUse` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class NestedArrayExtraUse extends java.lang.Object {
    static int trace;

    public NestedArrayExtraUse() {
        // @method <init>()V
        // @declaration a constructor of `NestedArrayExtraUse`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static int element(int arg0) {
        // @method element(I)I
        // @declaration a static method of `NestedArrayExtraUse`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        NestedArrayExtraUse.trace = NestedArrayExtraUse.trace * 10 + arg0;
        return arg0;
    }

    static int[][] build() {
        // @method build()[[I
        // @declaration a static method of `NestedArrayExtraUse`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        int[][] local0 = new int[1][];
        int[] local1 = new int[]{element(4)};
        int local2 = local1.length;
        local0[0] = local1;
        if (local2 != 1) {
            throw new java.lang.AssertionError();
        } else {
            return local0;
        }
    }
}

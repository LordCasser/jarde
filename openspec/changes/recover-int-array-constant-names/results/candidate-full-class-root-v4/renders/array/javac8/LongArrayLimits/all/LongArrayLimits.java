// jarde: presentation of `LongArrayLimits` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class LongArrayLimits extends java.lang.Object {
    private static final int ARRAY_SIZE = 4;

    public LongArrayLimits() {
        // @method <init>()V
        // @declaration a constructor of `LongArrayLimits`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public long[] test() {
        // @method test()[J
        // @declaration an instance method of `LongArrayLimits`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return new long[]{0L, 1L, 9223372036854775807L, -9223372036854775807L};
    }
}

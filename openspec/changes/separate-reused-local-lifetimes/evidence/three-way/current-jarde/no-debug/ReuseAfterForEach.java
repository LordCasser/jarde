// jarde: presentation of `ReuseAfterForEach` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class ReuseAfterForEach extends java.lang.Object {
    public ReuseAfterForEach() {
        // @method <init>()V
        // @declaration a constructor of `ReuseAfterForEach`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static int sumThenReuse(int[] arg0) {
        // @method sumThenReuse([I)I
        // @declaration a static method of `ReuseAfterForEach`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        int[] local2;
        int local3;
        local1 = 0;
        local2 = arg0;
        for (int local5 : local2) {
            local1 = local1 + local5;
        }
        int local2_2 = local1 + 1;
        local3 = local2_2 + 2;
        return local1 + local2_2 + local3;
    }
}

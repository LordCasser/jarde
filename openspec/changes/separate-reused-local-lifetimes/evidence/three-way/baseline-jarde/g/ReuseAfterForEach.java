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

    public static int sumThenReuse(int[] values) {
        // @method sumThenReuse([I)I
        // @declaration a static method of `ReuseAfterForEach`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int sum;
        int[] first;
        int second;
        sum = 0;
        first = values;
        for (int value : first) {
            sum = sum + value;
        }
        first = sum + 1;
        second = first + 2;
        return sum + first + second;
    }
}

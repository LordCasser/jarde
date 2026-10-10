// jarde: presentation of `MaskCondition` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class MaskCondition extends java.lang.Object {
    public MaskCondition() {
        // @method <init>()V
        // @declaration a constructor of `MaskCondition`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public int method3(int arg1, int arg2) {
        // @method method3(II)I
        // @declaration an instance method of `MaskCondition`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        if (arg1 + arg2 < 10) {
            return arg1;
        } else if ((arg1 & arg2) != 0) {
            return arg1 * arg2;
    } else {
            return arg2;
    }
    }
}

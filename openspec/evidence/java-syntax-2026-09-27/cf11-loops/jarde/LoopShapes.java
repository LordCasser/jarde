// jarde: presentation of `LoopShapes` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class LoopShapes extends java.lang.Object {
    public LoopShapes() {
        // @method <init>()V
        // @declaration a constructor of `LoopShapes`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static int nested(int limit) {
        // @method nested(I)I
        // @declaration a static method of `LoopShapes`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int total;
        int outer;
        total = 0;
        int inner;
        for (outer = 0; outer < limit; outer = outer + 1) {
            for (inner = 0; inner < outer; inner = inner + 1) {
                total = total + (outer + inner);
            }
        }
        return total;
    }

    public static int sequential(int limit) {
        // @method sequential(I)I
        // @declaration a static method of `LoopShapes`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int total;
        int local2;
        total = 0;
        local2 = 0;
        while (local2 < limit) {
            total = total + local2;
            local2 = local2 + 1;
        }
        local2 = limit;
        while (local2 > 0) {
            total = total + local2;
            local2 = local2 - 1;
        }
        return total;
    }

    public static void main(java.lang.String[] args) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `LoopShapes`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println(nested(5));
        java.lang.System.out.println(sequential(5));
        return;
    }
}

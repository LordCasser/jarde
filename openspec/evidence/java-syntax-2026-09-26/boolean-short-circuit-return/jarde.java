// jarde: presentation of `BoolValue` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class BoolValue extends java.lang.Object {
    static int calls;

    public BoolValue() {
        // @method <init>()V
        // @declaration a constructor of `BoolValue`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static boolean positive(int arg0) {
        // @method positive(I)Z
        // @declaration a static method of `BoolValue`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        BoolValue.calls = BoolValue.calls + 1;
        return arg0 > 0;
    }

    public static boolean and(int arg0, int arg1) {
        // @method and(II)Z
        // @declaration a static method of `BoolValue`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return (arg0 > 0 ? arg1 > 0 ? 1 : 0 : 0) % 2 != 0;
    }

    public static boolean or(int arg0, int arg1) {
        // @method or(II)Z
        // @declaration a static method of `BoolValue`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return (arg0 <= 0 ? arg1 > 0 ? 1 : 0 : 1) % 2 != 0;
    }

    public static boolean effectfulAnd(int arg0, int arg1) {
        // @method effectfulAnd(II)Z
        // @declaration a static method of `BoolValue`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return (positive(arg0) ? positive(arg1) ? 1 : 0 : 0) % 2 != 0;
    }

    public static boolean effectfulOr(int arg0, int arg1) {
        // @method effectfulOr(II)Z
        // @declaration a static method of `BoolValue`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return (!positive(arg0) ? positive(arg1) ? 1 : 0 : 1) % 2 != 0;
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `BoolValue`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        int local2;
        for (local1 = -1; local1 <= 1; local1 = local1 + 2) {
            for (local2 = -1; local2 <= 1; local2 = local2 + 2) {
                BoolValue.calls = 0;
                java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append(local1).append(",").append(local2).append(":").append(and(local1, local2)).append(",").append(or(local1, local2)).append(",").append(effectfulAnd(local1, local2)).append(",").append(effectfulOr(local1, local2)).append(",").append(BoolValue.calls).toString());
            }
        }
        return;
    }
}

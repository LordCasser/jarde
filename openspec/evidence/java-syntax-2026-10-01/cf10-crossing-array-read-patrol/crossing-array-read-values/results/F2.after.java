// jarde: presentation of `F2` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class F2 extends java.lang.Object {
    public F2() {
        // @method <init>()V
        // @declaration a constructor of `F2`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static int stepTwoWithQuote(int[] arg0) {
        // @method stepTwoWithQuote([I)I
        // @declaration a static method of `F2`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        int local2;
        local1 = 0;
        local2 = 0;
        while (local2 < arg0.length) {
            local1 = local1 + arg0[local2];
            try {
                local1 = local1 + risky(arg0[local2]);
            } catch (java.lang.IllegalStateException local3) {
                local1 = local1 - 1;
            }
            local2 = local2 + 2;
        }
        return local1;
    }

    static int risky(int arg0) {
        // @method risky(I)I
        // @declaration a static method of `F2`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        if (arg0 == 3) {
            throw new java.lang.IllegalStateException("r");
        } else {
            return arg0;
        }
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `F2`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int[] local1 = new int[]{1, 2, 3, 4, 5};
        java.lang.System.out.println(stepTwoWithQuote(local1));
        return;
    }
}

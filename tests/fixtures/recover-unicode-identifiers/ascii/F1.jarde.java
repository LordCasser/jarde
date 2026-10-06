// jarde: presentation of `F1` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class F1 extends java.lang.Object {
    public F1() {
        // @method <init>()V
        // @declaration a constructor of `F1`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static int stepTwo(int[] arg0) {
        // @method stepTwo([I)I
        // @declaration a static method of `F1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        int local2;
        local1 = 0;
        local2 = 0;
        while (local2 < arg0.length) {
            local1 = local1 + arg0[local2];
            local2 = local2 + 2;
        }
        return local1;
    }

    public static java.lang.String stepTwoWithCall(int[] arg0) {
        // @method stepTwoWithCall([I)Ljava/lang/String;
        // @declaration a static method of `F1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.StringBuilder local1;
        int local2;
        local1 = new java.lang.StringBuilder();
        local2 = 0;
        while (local2 < arg0.length) {
            local1.append(pick(arg0[local2]));
            local2 = local2 + 2;
        }
        return local1.toString();
    }

    static int pick(int arg0) {
        // @method pick(I)I
        // @declaration a static method of `F1`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return arg0 * 3;
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `F1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int[] local1 = new int[]{1, 2, 3, 4, 5};
        java.lang.System.out.println(stepTwo(local1));
        java.lang.System.out.println((java.lang.String) stepTwoWithCall(local1));
        return;
    }
}

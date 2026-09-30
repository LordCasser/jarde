// jarde: presentation of `V1` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class V1 extends java.lang.Object {
    public V1() {
        // @method <init>()V
        // @declaration a constructor of `V1`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static int side() {
        // @method side()I
        // @declaration a static method of `V1`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return 2;
    }

    public static java.lang.String three() {
        // @method three()Ljava/lang/String;
        // @declaration a static method of `V1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.StringBuilder local0;
        int[] local2;
        local0 = new java.lang.StringBuilder();
        int[] local1 = new int[]{side(), side() + 1, side() * 2};
        local2 = local1;
        for (int local5 : local2) {
            local0.append(local5).append(',');
        }
        boolean[] local1_2 = new boolean[3];
        local1_2[side() - 1] = true;
        local0.append(local1_2[1]);
        java.lang.Object[] local1_3 = new java.lang.Object[]{"x", "y"};
        local0.append(local1_3[1]);
        return local0.toString();
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `V1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) three());
        return;
    }
}

// jarde: presentation of `A2` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class A2 extends java.lang.Object {
    public A2() {
        // @method <init>()V
        // @declaration a constructor of `A2`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static int side() {
        // @method side()I
        // @declaration a static method of `A2`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return 2;
    }

    public static java.lang.String fillCalc() {
        // @method fillCalc()Ljava/lang/String;
        // @declaration a static method of `A2`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.StringBuilder local1;
        int[] local2;
        int[] local0 = new int[]{side(), side() + 1, side() * 2};
        local1 = new java.lang.StringBuilder();
        local2 = local0;
        for (int local5 : local2) {
            local1.append(local5).append(',');
        }
        local2 = new boolean[3];
        local2[side() - 1] = true;
        return new java.lang.StringBuilder().append((java.lang.String) local1.toString()).append(local2[1]).toString();
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `A2`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) fillCalc());
        return;
    }
}

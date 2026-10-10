// jarde: presentation of `A1` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class A1 extends java.lang.Object {
    static int calls = 0;

    public A1() {
        // @method <init>()V
        // @declaration a constructor of `A1`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static int n() {
        // @method n()I
        // @declaration a static method of `A1`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        A1.calls = A1.calls + 1;
        return 3;
    }

    public static int[][] dynDims() {
        // @method dynDims()[[I
        // @declaration a static method of `A1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int[][] local0 = new int[n()][];
        local0[0] = new int[n()];
        return new int[][]{local0[0], new int[n()]};
    }

    public static java.lang.Object[] mixed() {
        // @method mixed()[Ljava/lang/Object;
        // @declaration a static method of `A1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return new java.lang.Object[]{"s", java.lang.Integer.valueOf(n()), new int[n()]};
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `A1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int[][] local1 = dynDims();
        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append(local1.length).append(":").append(local1[0].length).append(":").append(A1.calls).toString());
        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append(mixed().length).append(":").append(A1.calls).toString());
        return;
    }
}

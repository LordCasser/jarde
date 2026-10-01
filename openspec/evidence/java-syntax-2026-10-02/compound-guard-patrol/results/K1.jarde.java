// jarde: presentation of `K1` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class K1 extends java.lang.Object {
    static int a;

    static java.lang.String b;

    static final int C;

    static int d;

    static int e;

    public K1() {
        // @method <init>()V
        // @declaration a constructor of `K1`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static int compute() {
        // @method compute()I
        // @declaration a static method of `K1`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return K1.a + 100;
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `K1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append(K1.a).append(":").append(K1.b).append(":").append(K1.C).append(":").append(K1.d).append(":").append(K1.e).toString());
        return;
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `K1`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        K1.a = 1;
        K1.a = K1.a + 10;
        if (K1.a > 5) {
            K1.b = "big";
        } else {
            K1.b = "small";
        }
        C = 42;
        K1.d = compute();
        try {
            K1.e = java.lang.Integer.parseInt("7");
        } catch (java.lang.NumberFormatException local0) {
            K1.e = -1;
        }
    }
}

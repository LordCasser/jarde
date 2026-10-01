// jarde: presentation of `K2` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class K2 extends java.lang.Object {
    static int a;

    static int b;

    public K2() {
        // @method <init>()V
        // @declaration a constructor of `K2`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static int sum() {
        // @method sum()I
        // @declaration a static method of `K2`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return K2.a + K2.b;
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `K2`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println(sum());
        return;
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `K2`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        K2.a = Other.first;
        K2.b = Other.second();
    }
}

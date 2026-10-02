// jarde: presentation of `IBRMulti` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class IBRMulti extends java.lang.Object {
    static int a;

    static int b;

    static int c;

    public IBRMulti() {
        // @method <init>()V
        // @declaration a constructor of `IBRMulti`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `IBRMulti`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append(IBRMulti.a).append(":").append(IBRMulti.b).append(":").append(IBRMulti.c).toString());
        return;
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `IBRMulti`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        IBRMulti.a = 1;
        IBRMulti.b = 2;
        if (java.lang.Boolean.getBoolean("ibr-multi-boom")) {
            throw new java.lang.IllegalStateException("multi-fail");
        } else {
            IBRMulti.a = IBRMulti.a + IBRMulti.b;
            IBRMulti.c = IBRMulti.a * 3;
            IBRMulti.b = IBRMulti.a + IBRMulti.c;
        }
    }
}

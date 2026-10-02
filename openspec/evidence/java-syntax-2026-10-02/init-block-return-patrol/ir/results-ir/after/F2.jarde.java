// jarde: presentation of `F2` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class F2 extends java.lang.Object {
    static int value;

    static int other;

    public F2() {
        // @method <init>()V
        // @declaration a constructor of `F2`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static int init() {
        // @method init()I
        // @declaration a static method of `F2`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        try {
            return java.lang.Integer.parseInt("42");
        } catch (java.lang.NumberFormatException local0) {
            throw new java.lang.RuntimeException("wrap", (java.lang.Throwable) local0);
        }
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `F2`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append(F2.value).append(":").append(F2.other).toString());
        return;
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `F2`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        if (java.lang.Boolean.getBoolean("boom")) {
            throw new java.lang.IllegalStateException("clinit-fail");
        } else {
            F2.value = 5;
            F2.other = init();
        }
    }
}

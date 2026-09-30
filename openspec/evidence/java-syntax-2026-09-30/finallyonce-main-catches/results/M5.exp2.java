// jarde: presentation of `M5` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class M5 extends java.lang.Object {
    public M5() {
        // @method <init>()V
        // @declaration a constructor of `M5`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static void helper() {
        // @method helper()V
        // @declaration a static method of `M5`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println("helper");
        return;
    }

    public static void t() {
        // @method t()V
        // @declaration a static method of `M5`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        throw new java.lang.IllegalStateException("state");
    }

    public static void u(java.lang.IllegalStateException arg0) {
        // @method u(Ljava/lang/IllegalStateException;)V
        // @declaration a static method of `M5`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) arg0.getMessage());
        return;
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `M5`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        helper();
        try {
            t();
            java.lang.System.out.println("missing throw");
        } catch (java.lang.IllegalStateException local1) {
            u(local1);
        }
        return;
    }
}

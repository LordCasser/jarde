// jarde: presentation of `P2` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class P2 extends java.lang.Object implements java.lang.AutoCloseable {
    static java.lang.StringBuilder log;

    static boolean mode;

    public P2() {
        // @method <init>()V
        // @declaration a constructor of `P2`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public void close() {
        // @method close()V
        // @declaration an instance method of `P2`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        P2.log.append("[c]");
        return;
    }

    public static java.lang.String branchNoCatch() throws java.lang.Exception {
        // jarde: not recovered: the recovery run for `branchNoCatch()Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method branchNoCatch()Ljava/lang/String;
        // @declaration a static method of `P2`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 14 24 40 48 54 56
        // local 0 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }

    public static java.lang.String plainCatch() throws java.lang.Exception {
        // @method plainCatch()Ljava/lang/String;
        // @declaration a static method of `P2`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        try (P2 local0 = new P2()) {
            P2.log.append("b");
        } catch (java.lang.IllegalStateException local0) {
            P2.log.append("E");
        }
        return P2.log.toString();
    }

    public static void main(java.lang.String[] arg0) throws java.lang.Exception {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `P2`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) branchNoCatch());
        java.lang.System.out.println((java.lang.String) plainCatch());
        return;
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `P2`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        P2.log = new java.lang.StringBuilder();
    }
}

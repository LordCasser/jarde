// jarde: presentation of `discardprobe/DiscardedCallSourceProbe` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package discardprobe;

public final class DiscardedCallSourceProbe extends java.lang.Object {
    private DiscardedCallSourceProbe() {
        // @method <init>()V
        // @declaration a constructor of `discardprobe.DiscardedCallSourceProbe`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static java.lang.String give(boolean arg0) {
        // @method give(Z)Ljava/lang/String;
        // @declaration a static method of `discardprobe.DiscardedCallSourceProbe`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        if (arg0) {
            throw new java.lang.IllegalStateException("give-failed");
        } else {
            return "given";
        }
    }

    public static void discardStatic(boolean arg0) {
        // @method discardStatic(Z)V
        // @declaration a static method of `discardprobe.DiscardedCallSourceProbe`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        give(arg0);
        return;
    }

    public static java.lang.String discardAppend(java.lang.String arg0) {
        // @method discardAppend(Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `discardprobe.DiscardedCallSourceProbe`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.StringBuilder local1 = new java.lang.StringBuilder();
        local1.append(arg0);
        return local1.toString();
    }

    public static java.lang.String discardListAdd(java.lang.String arg0) {
        // @method discardListAdd(Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `discardprobe.DiscardedCallSourceProbe`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.util.ArrayList local1 = new java.util.ArrayList();
        local1.add((java.lang.Object) arg0);
        return (java.lang.String) local1.get(0);
    }

    public static java.lang.String consumeReturn() {
        // @method consumeReturn()Ljava/lang/String;
        // @declaration a static method of `discardprobe.DiscardedCallSourceProbe`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return give(false);
    }

    public static java.lang.String deferToLocal() {
        // @method deferToLocal()Ljava/lang/String;
        // @declaration a static method of `discardprobe.DiscardedCallSourceProbe`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.String local0 = give(false);
        return local0;
    }
}

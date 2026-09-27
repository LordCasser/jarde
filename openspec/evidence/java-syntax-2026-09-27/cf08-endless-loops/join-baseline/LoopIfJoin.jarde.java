// jarde: presentation of `cf08join/LoopIfJoin` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package cf08join;

public final class LoopIfJoin extends java.lang.Object {
    public LoopIfJoin() {
        // @method <init>()V
        // @declaration a constructor of `cf08join.LoopIfJoin`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static int run(boolean arg0, boolean arg1) {
        // jarde: not recovered: the recovery run for `run(ZZ)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method run(ZZ)I
        // @declaration a static method of `cf08join.LoopIfJoin`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 6 10 15 20 26 32 34
        // local 2 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `cf08join.LoopIfJoin`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println(run(false, false));
        java.lang.System.out.println(run(true, false));
        java.lang.System.out.println(run(true, true));
        return;
    }
}

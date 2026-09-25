// jarde: presentation of `CatchNextScope` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class CatchNextScope extends java.lang.Object {
    public CatchNextScope() {
        // @method <init>()V
        // @declaration a constructor of `CatchNextScope`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static int consume(java.lang.Iterable values) {
        // jarde: not recovered: the recovery run for `consume(Ljava/lang/Iterable;)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method consume(Ljava/lang/Iterable;)I
        // @declaration a static method of `CatchNextScope`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 9 18 42 49
        // local 1 crosses a protected region, but SSA does not prove that every path to its reads reaches a presented write
    }
}

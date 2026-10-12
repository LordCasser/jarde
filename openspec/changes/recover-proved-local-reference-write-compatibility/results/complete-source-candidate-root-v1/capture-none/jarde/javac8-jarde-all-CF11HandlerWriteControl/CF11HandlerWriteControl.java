// jarde: presentation of `cf11/CF11HandlerWriteControl` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package cf11;

public final class CF11HandlerWriteControl extends java.lang.Object {
    public CF11HandlerWriteControl() {
        // @method <init>()V
        // @declaration a constructor of `cf11.CF11HandlerWriteControl`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    private static java.lang.Object asObject(java.lang.Object arg0) {
        // @method asObject(Ljava/lang/Object;)Ljava/lang/Object;
        // @declaration a static method of `cf11.CF11HandlerWriteControl`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        return arg0;
    }

    public static java.lang.Object handlerWrite() {
        // jarde: not recovered: the recovery run for `handlerWrite()Ljava/lang/Object;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method handlerWrite()Ljava/lang/Object;
        // @declaration a static method of `cf11.CF11HandlerWriteControl`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 13
        // local 0 crosses a protected region, but SSA does not prove that every path to its reads reaches a presented write
    }
}

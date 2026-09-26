// jarde: presentation of `LoopBool` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class LoopBool extends java.lang.Object {
    public LoopBool() {
        // @method <init>()V
        // @declaration a constructor of `LoopBool`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static int andWhile(int arg0, int arg1) {
        // @method andWhile(II)I
        // @declaration a static method of `LoopBool`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local2;
        local2 = 0;
        while (arg0 > 0 && arg1 > 0) {
            local2 = local2 + arg0;
            arg0 = arg0 - 1;
            arg1 = arg1 - 1;
        }
        return local2;
    }

    public static int orWhile(int arg0, int arg1) {
        // @method orWhile(II)I
        // @declaration a static method of `LoopBool`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local2;
        local2 = 0;
        while (arg0 > 0 || arg1 > 0) {
            local2 = local2 + 1;
            arg0 = arg0 - 1;
            arg1 = arg1 - 1;
        }
        return local2;
    }

    public static int mixedWhile(int arg0, int arg1) {
        // jarde: not recovered: the recovery run for `mixedWhile(II)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method mixedWhile(II)I
        // @declaration a static method of `LoopBool`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 2 6 10 15 27
        // local 2 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }
}

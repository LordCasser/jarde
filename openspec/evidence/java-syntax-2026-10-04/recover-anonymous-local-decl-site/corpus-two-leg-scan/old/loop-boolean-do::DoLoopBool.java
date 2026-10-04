// jarde: presentation of `DoLoopBool` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class DoLoopBool extends java.lang.Object {
    public DoLoopBool() {
        // @method <init>()V
        // @declaration a constructor of `DoLoopBool`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static int andDo(int arg0, int arg1) {
        // @method andDo(II)I
        // @declaration a static method of `DoLoopBool`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local2;
        local2 = 0;
        do {
            local2 = local2 + 1;
            arg0 = arg0 - 1;
            arg1 = arg1 - 1;
        } while (arg0 > 0 && arg1 > 0);
        return local2;
    }

    public static int orDo(int arg0, int arg1) {
        // @method orDo(II)I
        // @declaration a static method of `DoLoopBool`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local2;
        local2 = 0;
        do {
            local2 = local2 + 1;
            arg0 = arg0 - 1;
            arg1 = arg1 - 1;
        } while (arg0 > 0 || arg1 > 0);
        return local2;
    }
}

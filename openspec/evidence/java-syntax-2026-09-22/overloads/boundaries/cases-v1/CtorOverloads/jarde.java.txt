// jarde: presentation of `CtorOverloads` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class CtorOverloads extends java.lang.Object {
    private final int code;

    public CtorOverloads(java.lang.Object arg1) {
        // @method <init>(Ljava/lang/Object;)V
        // @declaration a constructor of `CtorOverloads`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.code = 1;
        return;
    }

    public CtorOverloads(java.lang.String arg1) {
        // @method <init>(Ljava/lang/String;)V
        // @declaration a constructor of `CtorOverloads`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.code = 2;
        return;
    }

    public static int run() {
        // @method run()I
        // @declaration a static method of `CtorOverloads`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return new CtorOverloads(null).code;
    }
}

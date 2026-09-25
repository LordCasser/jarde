// jarde: presentation of `MultiArgs` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class MultiArgs extends java.lang.Object {
    public MultiArgs() {
        // @method <init>()V
        // @declaration a constructor of `MultiArgs`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static int pick(java.lang.Object arg0, java.lang.Object arg1) {
        // @method pick(Ljava/lang/Object;Ljava/lang/Object;)I
        // @declaration a static method of `MultiArgs`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return 1;
    }

    public static int pick(java.lang.String arg0, java.lang.String arg1) {
        // @method pick(Ljava/lang/String;Ljava/lang/String;)I
        // @declaration a static method of `MultiArgs`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return 2;
    }

    public static int run() {
        // @method run()I
        // @declaration a static method of `MultiArgs`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return pick(null, "x");
    }
}

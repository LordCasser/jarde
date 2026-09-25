// jarde: presentation of `TernaryCore` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class TernaryCore extends java.lang.Object {
    private static int trace;

    public TernaryCore() {
        // @method <init>()V
        // @declaration a constructor of `TernaryCore`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static void reset() {
        // @method reset()V
        // @declaration a static method of `TernaryCore`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        TernaryCore.trace = 0;
        return;
    }

    public static int trace() {
        // @method trace()I
        // @declaration a static method of `TernaryCore`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return TernaryCore.trace;
    }

    private static int a() {
        // @method a()I
        // @declaration a static method of `TernaryCore`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        TernaryCore.trace = TernaryCore.trace * 10 + 1;
        return 7;
    }

    private static int b() {
        // @method b()I
        // @declaration a static method of `TernaryCore`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        TernaryCore.trace = TernaryCore.trace * 10 + 2;
        return 11;
    }

    public static int returned(boolean arg0) {
        // @method returned(Z)I
        // @declaration a static method of `TernaryCore`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return arg0 ? a() : b();
    }
}

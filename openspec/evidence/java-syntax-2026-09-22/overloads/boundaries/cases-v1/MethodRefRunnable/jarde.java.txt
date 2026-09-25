// jarde: presentation of `MethodRefRunnable` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class MethodRefRunnable extends java.lang.Object {
    public MethodRefRunnable() {
        // @method <init>()V
        // @declaration a constructor of `MethodRefRunnable`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static int action(java.lang.Runnable arg0) {
        // @method action(Ljava/lang/Runnable;)I
        // @declaration a static method of `MethodRefRunnable`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return 7;
    }

    public static int action(java.util.function.Supplier arg0) {
        // @method action(Ljava/util/function/Supplier;)I
        // @declaration a static method of `MethodRefRunnable`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return 8;
    }

    public static int run() {
        // @method run()I
        // @declaration a static method of `MethodRefRunnable`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return action(java.lang.System::nanoTime);
    }
}

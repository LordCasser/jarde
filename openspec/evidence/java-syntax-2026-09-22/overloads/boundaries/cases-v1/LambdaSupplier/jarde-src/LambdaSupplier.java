// jarde: presentation of `LambdaSupplier` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class LambdaSupplier extends java.lang.Object {
    public LambdaSupplier() {
        // @method <init>()V
        // @declaration a constructor of `LambdaSupplier`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static int action(java.lang.Runnable arg0) {
        // @method action(Ljava/lang/Runnable;)I
        // @declaration a static method of `LambdaSupplier`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return 7;
    }

    public static int action(java.util.function.Supplier arg0) {
        // @method action(Ljava/util/function/Supplier;)I
        // @declaration a static method of `LambdaSupplier`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return 8;
    }

    public static int run() {
        // @method run()I
        // @declaration a static method of `LambdaSupplier`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return action(() -> LambdaSupplier.lambda$run$0());
    }

    private static java.lang.String lambda$run$0() {
        // @method lambda$run$0()Ljava/lang/String;
        // @declaration a static method of `LambdaSupplier`, member flags 0x100a
        // recovered from bytecode; presentation is not claimed to compile
        return "s";
    }
}

// jarde: presentation of `OverloadEdges` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class OverloadEdges extends java.lang.Object {
    public OverloadEdges() {
        // @method <init>()V
        // @declaration a constructor of `OverloadEdges`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static int choose(java.lang.Object arg0) {
        // @method choose(Ljava/lang/Object;)I
        // @declaration a static method of `OverloadEdges`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return 1;
    }

    public static int choose(java.lang.String arg0) {
        // @method choose(Ljava/lang/String;)I
        // @declaration a static method of `OverloadEdges`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return 2;
    }

    public static int arr(java.lang.Object arg0) {
        // @method arr(Ljava/lang/Object;)I
        // @declaration a static method of `OverloadEdges`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return 3;
    }

    public static int arr(java.lang.String[] arg0) {
        // @method arr([Ljava/lang/String;)I
        // @declaration a static method of `OverloadEdges`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return 4;
    }

    public static int boxed(java.lang.Object arg0) {
        // @method boxed(Ljava/lang/Object;)I
        // @declaration a static method of `OverloadEdges`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return 5;
    }

    public static int boxed(java.lang.Integer arg0) {
        // @method boxed(Ljava/lang/Integer;)I
        // @declaration a static method of `OverloadEdges`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return 6;
    }

    public static int action(java.lang.Runnable arg0) {
        // @method action(Ljava/lang/Runnable;)I
        // @declaration a static method of `OverloadEdges`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return 7;
    }

    public static int action(java.util.function.Supplier arg0) {
        // @method action(Ljava/util/function/Supplier;)I
        // @declaration a static method of `OverloadEdges`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return 8;
    }

    public static int nullObject() {
        // @method nullObject()I
        // @declaration a static method of `OverloadEdges`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return choose(null);
    }

    public static int arrayObject(java.lang.String[] arg0) {
        // @method arrayObject([Ljava/lang/String;)I
        // @declaration a static method of `OverloadEdges`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return arr(arg0);
    }

    public static int boxObject(int arg0) {
        // @method boxObject(I)I
        // @declaration a static method of `OverloadEdges`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return boxed(java.lang.Integer.valueOf(arg0));
    }

    public static int lambdaRunnable() {
        // @method lambdaRunnable()I
        // @declaration a static method of `OverloadEdges`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return action(() -> OverloadEdges.lambda$lambdaRunnable$0());
    }

    public static int lambdaSupplier() {
        // @method lambdaSupplier()I
        // @declaration a static method of `OverloadEdges`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return action(() -> OverloadEdges.lambda$lambdaSupplier$1());
    }

    public static int methodRefRunnable() {
        // @method methodRefRunnable()I
        // @declaration a static method of `OverloadEdges`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return action(java.lang.System::nanoTime);
    }

    private static java.lang.String lambda$lambdaSupplier$1() {
        // @method lambda$lambdaSupplier$1()Ljava/lang/String;
        // @declaration a static method of `OverloadEdges`, member flags 0x100a
        // recovered from bytecode; presentation is not claimed to compile
        return "s";
    }

    private static void lambda$lambdaRunnable$0() {
        // @method lambda$lambdaRunnable$0()V
        // @declaration a static method of `OverloadEdges`, member flags 0x100a
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.nanoTime();
        // @bytecode 3
        // the instruction at BCI 3 is not part of the provable subset
        return;
    }
}

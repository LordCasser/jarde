// jarde: presentation of `BoundFunctionalReceiver` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class BoundFunctionalReceiver extends java.lang.Object {
    public BoundFunctionalReceiver() {
        // @method <init>()V
        // @declaration a constructor of `BoundFunctionalReceiver`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static int array(int n) {
        // @method array(I)I
        // @declaration a static method of `BoundFunctionalReceiver`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.util.function.IntFunction f = (int p0) -> BoundFunctionalReceiver.lambda$array$0(p0);
        return ((int[]) f.apply(n)).length;
    }

    public static int method(int n) {
        // @method method(I)I
        // @declaration a static method of `BoundFunctionalReceiver`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.util.function.IntUnaryOperator f = java.lang.Math::abs;
        return f.applyAsInt(n);
    }

    private static int[] lambda$array$0(int x$0) {
        // @method lambda$array$0(I)[I
        // @declaration a static method of `BoundFunctionalReceiver`, member flags 0x100a
        // recovered from bytecode; presentation is not claimed to compile
        return new int[x$0];
    }
}

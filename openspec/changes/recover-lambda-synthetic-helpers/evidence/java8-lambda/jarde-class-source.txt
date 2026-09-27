// jarde: presentation of `LambdaFixture` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class LambdaFixture extends java.lang.Object {
    public LambdaFixture() {
        // @method <init>()V
        // @declaration a constructor of `LambdaFixture`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static java.util.function.IntSupplier zero() {
        // @method zero()Ljava/util/function/IntSupplier;
        // @declaration a static method of `LambdaFixture`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return () -> LambdaFixture.lambda$zero$0();
    }

    public static java.util.function.IntUnaryOperator one() {
        // @method one()Ljava/util/function/IntUnaryOperator;
        // @declaration a static method of `LambdaFixture`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return (int p0) -> LambdaFixture.lambda$one$1(p0);
    }

    public static java.util.function.IntBinaryOperator two() {
        // @method two()Ljava/util/function/IntBinaryOperator;
        // @declaration a static method of `LambdaFixture`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return (int p0, int p1) -> LambdaFixture.lambda$two$2(p0, p1);
    }

    private static int lambda$two$2(int arg0, int arg1) {
        // @method lambda$two$2(II)I
        // @declaration a static method of `LambdaFixture`, member flags 0x100a
        // recovered from bytecode; presentation is not claimed to compile
        return arg0 * 10 + arg1;
    }

    private static int lambda$one$1(int arg0) {
        // @method lambda$one$1(I)I
        // @declaration a static method of `LambdaFixture`, member flags 0x100a
        // recovered from bytecode; presentation is not claimed to compile
        return arg0 + 10;
    }

    private static int lambda$zero$0() {
        // @method lambda$zero$0()I
        // @declaration a static method of `LambdaFixture`, member flags 0x100a
        // recovered from bytecode; presentation is not claimed to compile
        return 7;
    }
}

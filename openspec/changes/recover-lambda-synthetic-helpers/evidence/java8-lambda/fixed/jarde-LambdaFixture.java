// jarde: presentation of `LambdaFixture` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class LambdaFixture extends java.lang.Object {
    // jarde: omitted physical lambda helper JvmBytes([108, 97, 109, 98, 100, 97, 36, 122, 101, 114, 111, 36, 48]) after proving all class-wide uses
    // jarde: omitted physical lambda helper JvmBytes([108, 97, 109, 98, 100, 97, 36, 111, 110, 101, 36, 49]) after proving all class-wide uses
    // jarde: omitted physical lambda helper JvmBytes([108, 97, 109, 98, 100, 97, 36, 116, 119, 111, 36, 50]) after proving all class-wide uses
    public LambdaFixture() {
        // @method <init>()V
        // @declaration a constructor of `LambdaFixture`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static java.util.function.IntSupplier zero() {
        // jarde: inlined exact no-capture primitive lambda helper JvmBytes([108, 97, 109, 98, 100, 97, 36, 122, 101, 114, 111, 36, 48]) at invokedynamic@0
        // @method zero()Ljava/util/function/IntSupplier;
        // @declaration a static method of `LambdaFixture`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return () -> 7;
    }

    public static java.util.function.IntUnaryOperator one() {
        // jarde: inlined exact no-capture primitive lambda helper JvmBytes([108, 97, 109, 98, 100, 97, 36, 111, 110, 101, 36, 49]) at invokedynamic@0
        // @method one()Ljava/util/function/IntUnaryOperator;
        // @declaration a static method of `LambdaFixture`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return (int p0) -> p0 + 10;
    }

    public static java.util.function.IntBinaryOperator two() {
        // jarde: inlined exact no-capture primitive lambda helper JvmBytes([108, 97, 109, 98, 100, 97, 36, 116, 119, 111, 36, 50]) at invokedynamic@0
        // @method two()Ljava/util/function/IntBinaryOperator;
        // @declaration a static method of `LambdaFixture`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return (int p0, int p1) -> p0 * 10 + p1;
    }
}

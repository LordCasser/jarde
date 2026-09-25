// jarde: presentation of `LambdaAlias` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class LambdaAlias extends java.lang.Object {
    private final int factor;

    public LambdaAlias(int factor) {
        // @method <init>(I)V
        // @declaration a constructor of `LambdaAlias`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.factor = factor;
        return;
    }

    public java.util.function.IntUnaryOperator build(int base) {
        // @method build(I)Ljava/util/function/IntUnaryOperator;
        // @declaration an instance method of `LambdaAlias`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return (int p0) -> this.lambda$build$0(base, p0);
    }

    public int direct(int left, int right) {
        // @method direct(II)I
        // @declaration an instance method of `LambdaAlias`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.lambda$build$0(left, right);
    }

    private int directHelper(int left, int right) {
        // @method directHelper(II)I
        // @declaration an instance method of `LambdaAlias`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        int sum = left + right;
        return sum * this.factor + 100;
    }

    private int lambda$build$0(int base, int value) {
        // @method lambda$build$0(II)I
        // @declaration an instance method of `LambdaAlias`, member flags 0x1002
        // recovered from bytecode; presentation is not claimed to compile
        int scaled = value * this.factor;
        int adjusted = scaled + base;
        return adjusted - 3;
    }
}

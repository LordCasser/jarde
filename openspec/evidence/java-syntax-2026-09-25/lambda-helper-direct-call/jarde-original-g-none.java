// jarde: presentation of `LambdaAlias` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class LambdaAlias extends java.lang.Object {
    private final int factor;

    public LambdaAlias(int arg1) {
        // @method <init>(I)V
        // @declaration a constructor of `LambdaAlias`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.factor = arg1;
        return;
    }

    public java.util.function.IntUnaryOperator build(int arg1) {
        // @method build(I)Ljava/util/function/IntUnaryOperator;
        // @declaration an instance method of `LambdaAlias`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return (int p0) -> this.lambda$build$0(arg1, p0);
    }

    public int direct(int arg1, int arg2) {
        // @method direct(II)I
        // @declaration an instance method of `LambdaAlias`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.directHelper(arg1, arg2);
    }

    private int directHelper(int arg1, int arg2) {
        // @method directHelper(II)I
        // @declaration an instance method of `LambdaAlias`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        int local3 = arg1 + arg2;
        return local3 * this.factor + 100;
    }

    private int lambda$build$0(int arg1, int arg2) {
        // @method lambda$build$0(II)I
        // @declaration an instance method of `LambdaAlias`, member flags 0x1002
        // recovered from bytecode; presentation is not claimed to compile
        int local3 = arg2 * this.factor;
        int local4 = local3 + arg1;
        return local4 - 3;
    }
}

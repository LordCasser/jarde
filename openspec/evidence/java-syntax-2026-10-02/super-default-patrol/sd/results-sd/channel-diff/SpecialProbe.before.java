// jarde: presentation of `SpecialProbe` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class SpecialProbe extends BaseProbe implements DefaultProbe {
    private final int bias;

    public SpecialProbe() {
        // @method <init>()V
        // @declaration a constructor of `SpecialProbe`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super(7);
        this.bias = 2;
        return;
    }

    public SpecialProbe(int arg1) {
        // @method <init>(I)V
        // @declaration a constructor of `SpecialProbe`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super(7);
        this.bias = arg1;
        return;
    }

    @Override
    public int value() {
        // @method value()I
        // @declaration an instance method of `SpecialProbe`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return super.value() + 1;
    }

    public int defaultCall() {
        // @method defaultCall()I
        // @declaration an instance method of `SpecialProbe`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return DefaultProbe.super.value();
    }

    private int privateHelper(int arg1) {
        // @method privateHelper(I)I
        // @declaration an instance method of `SpecialProbe`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        return arg1 + this.bias;
    }

    public int callOwnPrivate(int arg1) {
        // @method callOwnPrivate(I)I
        // @declaration an instance method of `SpecialProbe`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.privateHelper(arg1);
    }

    public int callOtherPrivate(SpecialProbe arg1, int arg2) {
        // @method callOtherPrivate(LSpecialProbe;I)I
        // @declaration an instance method of `SpecialProbe`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return arg1.privateHelper(arg2);
    }

    public int superWithSideEffect() {
        // @method superWithSideEffect()I
        // @declaration an instance method of `SpecialProbe`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return super.valueWith(BaseProbe.sideEffectArgument());
    }

    public int superWithThrowingArgument() {
        // @method superWithThrowingArgument()I
        // @declaration an instance method of `SpecialProbe`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return super.valueWith(BaseProbe.throwingArgument());
    }

    public int superThrowing() {
        // @method superThrowing()I
        // @declaration an instance method of `SpecialProbe`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return super.failWith(3);
    }
}

public class SpecialProbe extends BaseProbe implements DefaultProbe {
    private final int bias;

    public SpecialProbe() {
        super(7);
        this.bias = 2;
    }

    public SpecialProbe(int bias) {
        super(7);
        this.bias = bias;
    }

    @Override
    public int value() {
        return super.value() + 1;
    }

    public int defaultCall() {
        return DefaultProbe.super.value();
    }

    private int privateHelper(int value) {
        return value + bias;
    }

    public int callOwnPrivate(int value) {
        return privateHelper(value);
    }

    public int callOtherPrivate(SpecialProbe other, int value) {
        return other.privateHelper(value);
    }

    public int superWithSideEffect() {
        return super.valueWith(BaseProbe.sideEffectArgument());
    }

    public int superWithThrowingArgument() {
        return super.valueWith(BaseProbe.throwingArgument());
    }

    public int superThrowing() {
        return super.failWith(3);
    }

}

public class SpecialProbe extends BaseProbe implements DefaultProbe {
    @Override
    public int value() {
        return super.value() + 1;
    }

    public int defaultCall() {
        return DefaultProbe.super.value();
    }

    private int privateHelper(int value) {
        return value + 2;
    }

    public int callOwnPrivate(int value) {
        return privateHelper(value);
    }

    public int callOtherPrivate(SpecialProbe other, int value) {
        return other.privateHelper(value);
    }
}

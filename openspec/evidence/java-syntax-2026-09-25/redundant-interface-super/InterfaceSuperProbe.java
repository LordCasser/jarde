class DefaultBase {
    public int value() { return 7; }
}

interface DefaultLeft {
    default int value() { return 11; }
}

interface DefaultRight {
    default int value() { return 22; }
}

public final class InterfaceSuperProbe extends DefaultBase implements DefaultLeft, DefaultRight {
    @Override public int value() { return DefaultLeft.super.value(); }
    public int chooseRight() { return DefaultRight.super.value(); }
    public int both() { return DefaultLeft.super.value() + DefaultRight.super.value(); }
}

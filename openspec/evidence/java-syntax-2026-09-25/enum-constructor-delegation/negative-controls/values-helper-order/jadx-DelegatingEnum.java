package defpackage;

/* JADX INFO: loaded from: input.jar:DelegatingEnum.class */
public enum DelegatingEnum {
    ONE(1),
    ZERO;

    private final int value;

    DelegatingEnum() {
        this(0);
    }

    DelegatingEnum(int i) {
        ConstructorEffects.record(i);
        this.value = i;
    }

    public int value() {
        return this.value;
    }
}

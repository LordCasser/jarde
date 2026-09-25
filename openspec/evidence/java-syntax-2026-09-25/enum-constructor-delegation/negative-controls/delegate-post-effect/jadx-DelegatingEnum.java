package defpackage;

/* JADX INFO: loaded from: input.jar:DelegatingEnum.class */
public enum DelegatingEnum {
    ZERO,
    ONE(1);

    private final int value;

    DelegatingEnum() {
        this(0);
        ConstructorEffects.record(9);
    }

    DelegatingEnum(int i) {
        ConstructorEffects.record(i);
        this.value = i;
    }

    public int value() {
        return this.value;
    }
}

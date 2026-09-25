package defpackage;

/* JADX INFO: loaded from: input.jar:DelegatingEnum.class */
public enum DelegatingEnum {
    ZERO,
    ONE(1);

    private final int value;

    DelegatingEnum() {
        this(0, 7);
    }

    DelegatingEnum(int i) {
        ConstructorEffects.record(i);
        this.value = i;
    }

    DelegatingEnum(int i, int i2) {
        ConstructorEffects.record(i + i2);
        this.value = i + i2;
    }

    public int value() {
        return this.value;
    }
}

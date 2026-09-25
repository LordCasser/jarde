package defpackage;

/* JADX INFO: loaded from: input.jar:DelegatingEnum.class */
public enum DelegatingEnum {
    ZERO,
    ONE(1);

    private final int value;

    DelegatingEnum() {
        this(0L);
    }

    DelegatingEnum(int i) {
        ConstructorEffects.record(i);
        this.value = i;
    }

    DelegatingEnum(long j) {
        ConstructorEffects.record((int) j);
        this.value = (int) j;
    }

    public int value() {
        return this.value;
    }
}

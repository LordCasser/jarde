
/* JADX INFO: loaded from: ConstructorConditionalProbe.class */
public final class ConstructorConditionalProbe {
    private final int value;

    public ConstructorConditionalProbe(java.lang.String str, int i) {
        this(str == null ? 0 : i);
    }

    public ConstructorConditionalProbe(int i) {
        this.value = i;
    }

    public int value() {
        return this.value;
    }
}

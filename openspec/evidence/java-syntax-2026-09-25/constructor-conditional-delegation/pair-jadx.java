
/* JADX INFO: loaded from: ConstructorPairProbe.class */
public final class ConstructorPairProbe {
    private final java.lang.String first;
    private final java.lang.String second;

    public ConstructorPairProbe(java.lang.String str, int i) {
        this(i == 1 ? str : "", i == 0 ? "" : str);
    }

    public ConstructorPairProbe(java.lang.String str, java.lang.String str2) {
        this.first = str;
        this.second = str2;
    }

    public java.lang.String first() {
        return this.first;
    }

    public java.lang.String second() {
        return this.second;
    }
}

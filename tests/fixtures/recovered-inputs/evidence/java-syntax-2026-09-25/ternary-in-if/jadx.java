
/* JADX INFO: loaded from: TernaryInIfProbe.class */
public final class TernaryInIfProbe {
    private final java.lang.String a;
    private final java.lang.String b;

    public TernaryInIfProbe(java.lang.String str, java.lang.String str2) {
        this.a = str;
        this.b = str2;
    }

    public boolean bothMatch(TernaryInIfProbe ternaryInIfProbe) {
        if (this.a == null) {
            if (ternaryInIfProbe.a != null) {
                return false;
            }
        } else if (!this.a.equals(ternaryInIfProbe.a)) {
            return false;
        }
        if (this.b == null) {
            return ternaryInIfProbe.b == null;
        }
        return this.b.equals(ternaryInIfProbe.b);
    }
}

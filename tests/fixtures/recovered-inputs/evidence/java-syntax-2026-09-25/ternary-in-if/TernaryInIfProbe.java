public final class TernaryInIfProbe {
    private final String a;
    private final String b;

    public TernaryInIfProbe(String a, String b) {
        this.a = a;
        this.b = b;
    }

    public boolean bothMatch(TernaryInIfProbe other) {
        if (this.a == null ? other.a == null : this.a.equals(other.a)) {
            if (this.b == null ? other.b == null : this.b.equals(other.b)) {
                return true;
            }
        }
        return false;
    }
}

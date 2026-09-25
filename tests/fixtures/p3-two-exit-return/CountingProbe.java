public final class CountingProbe {
    private final CountedValue a;
    private final CountedValue b;

    public CountingProbe(CountedValue a, CountedValue b) { this.a = a; this.b = b; }

    public boolean bothMatch(CountingProbe other) {
        if (a == null ? other.a == null : a.equals(other.a)) {
            if (b == null ? other.b == null : b.equals(other.b)) {
                return true;
            }
        }
        return false;
    }
}

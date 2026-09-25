public final class EffectProbe {
    private final String a;
    private final String b;

    public EffectProbe(String a, String b) {
        this.a = a;
        this.b = b;
    }

    public boolean bothMatch(EffectProbe other) {
        if (a == null ? other.a == null : a.equals(other.a)) {
            System.nanoTime();
            if (b == null ? other.b == null : b.equals(other.b)) {
                return true;
            }
        }
        return false;
    }
}

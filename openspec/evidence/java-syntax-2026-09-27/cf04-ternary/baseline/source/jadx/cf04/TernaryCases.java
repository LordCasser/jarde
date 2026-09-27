package cf04;

/* JADX INFO: loaded from: input.jar:cf04/TernaryCases.class */
public class TernaryCases {
    public static int calls;
    private final int value;

    public TernaryCases(String str, int i) {
        this(str == null ? 0 : i);
    }

    public TernaryCases(int i) {
        this.value = i;
    }

    public TernaryCases(String str, int i, boolean z) {
        this(i == 1 ? str : "", i == 0 ? "" : str);
    }

    public TernaryCases(String str, String str2) {
        this.value = (str.length() * 10) + str2.length();
    }

    public int value() {
        return this.value;
    }

    public static int positive(int i) {
        return i > 0 ? i : (i + 2) * 3;
    }

    public static boolean choose(boolean z, boolean z2, boolean z3) {
        return z ? z2 : z3;
    }

    public static int nested(boolean z, boolean z2, boolean z3) {
        return (z ? !z2 : !z3) ? 2 : 1;
    }

    private static int arm(int i) {
        calls++;
        return i;
    }

    public static int effect(boolean z) {
        return z ? arm(1) : arm(2);
    }
}

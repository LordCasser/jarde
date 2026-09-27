package cf04;

/* JADX INFO: loaded from: basic.jar:cf04/TernaryBasic.class */
public class TernaryBasic {
    public static int calls;
    private final int value;

    public TernaryBasic(String str, int i) {
        this(str == null ? 0 : i);
    }

    public TernaryBasic(int i) {
        this.value = i;
    }

    public TernaryBasic(String str, int i, boolean z) {
        this(i == 1 ? str : "", i == 0 ? "" : str);
    }

    public TernaryBasic(String str, String str2) {
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

    private static int arm(int i) {
        calls++;
        return i;
    }

    public static int effect(boolean z) {
        return z ? arm(1) : arm(2);
    }
}

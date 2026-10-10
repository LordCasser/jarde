package defpackage;

/* JADX INFO: loaded from: LocalSourceTypesBoundaries.class */
public final class LocalSourceTypesBoundaries {
    private static final String CHAR_INPUT = "./?A";
    private static char MUTABLE_CHAR_FIELD = 'F';

    private LocalSourceTypesBoundaries() {
    }

    public static String charCallAndLiteralWrites(int i, boolean z, boolean z2) {
        String str;
        char cCharAt = CHAR_INPUT.charAt(i);
        if (z) {
            cCharAt = 0;
        }
        if (z2) {
            cCharAt = 65535;
        }
        StringBuilder sb = new StringBuilder();
        sb.append(cCharAt);
        switch (cCharAt) {
            case 0:
                str = "zero";
                break;
            case '.':
                str = "dot";
                break;
            case '/':
                str = "slash";
                break;
            case '?':
                str = "question";
                break;
            default:
                str = "other";
                break;
        }
        return str + ":" + ((int) cCharAt) + ":" + sb.length();
    }

    public static String charFieldSeed() {
        char c = MUTABLE_CHAR_FIELD;
        StringBuilder sb = new StringBuilder();
        sb.append(c);
        return ((int) c) + ":" + sb.length();
    }

    public static int charI2cSeed(int i) {
        char c = (char) i;
        StringBuilder sb = new StringBuilder();
        sb.append(c);
        return c + sb.length();
    }

    public static String charEntryParameterSeed(char c, boolean z) {
        char c2 = c;
        if (z) {
            c2 = 65535;
        }
        StringBuilder sb = new StringBuilder();
        sb.append(c2);
        return ((int) c2) + ":" + sb.length();
    }

    public static int intWithOutOfRangeWrites(int i, int i2) {
        int i3;
        CHAR_INPUT.charAt(0);
        if (i == 0) {
            i3 = -1;
        } else {
            i3 = i == 1 ? 65536 : i2;
        }
        return i3;
    }

    public static int intWithArithmeticWrite(boolean z) {
        return z ? CHAR_INPUT.charAt(0) + 1 : 65535;
    }

    public static int intWithUnknownCopyMerge(boolean z, int i) {
        CHAR_INPUT.charAt(0);
        return z ? i : 65;
    }

    public static void exactStringWritesAfterNull(int i) {
        String str = null;
        switch (i) {
            case 1:
                str = "one";
                break;
            case 2:
                str = new String("two");
                break;
            case 3:
                str = "three";
                break;
        }
        System.out.println(str);
    }

    public static void mixedReferenceWrites(boolean z) {
        System.out.println(z ? "text" : new StringBuilder("builder"));
    }

    public static void allNullWrites(boolean z) {
        System.out.println(z ? null : null);
    }

    private static Object opaqueCopy(Object obj) {
        return obj;
    }

    public static void unknownReferenceCopy(boolean z, Object obj) {
        System.out.println(z ? opaqueCopy(obj) : "known");
    }

    public static String possibleSlotReuse(boolean z) {
        return Integer.toString(z ? 65536 : -1) + ":" + (z ? "reused-string" : new String("other-string"));
    }
}

package defpackage;

/* JADX INFO: loaded from: NestedConditional.class */
public final class NestedConditional {
    private NestedConditional() {
    }

    public static int nested(int i) {
        int i2;
        if (i > 10) {
            i2 = i > 100 ? 3 : 2;
        } else {
            i2 = 1;
        }
        return i2;
    }

    public static int nestedEffects(int i, StringBuilder sb) {
        int iArm;
        if (i <= outerLimit(i, sb)) {
            iArm = arm(sb, "1", 1, false);
        } else if (i > innerLimit(i, sb)) {
            iArm = arm(sb, "3", 3, false);
        } else {
            iArm = arm(sb, "2", 2, i == 12);
        }
        return iArm;
    }

    private static int outerLimit(int i, StringBuilder sb) {
        sb.append('O');
        if (i == 0) {
            throw new IllegalArgumentException("outer");
        }
        return 10;
    }

    private static int innerLimit(int i, StringBuilder sb) {
        sb.append('I');
        if (i == 50) {
            throw new IllegalStateException("inner");
        }
        return 100;
    }

    private static int arm(StringBuilder sb, String str, int i, boolean z) {
        sb.append(str);
        if (z) {
            throw new ArithmeticException("arm-" + str);
        }
        return i;
    }
}

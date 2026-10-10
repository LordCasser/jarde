package defpackage;

/* JADX INFO: loaded from: input.jar:ConditionalSwitchBoundaries.class */
public final class ConditionalSwitchBoundaries {
    private ConditionalSwitchBoundaries() {
    }

    public static String partialBreak(int i, int i2) {
        StringBuilder sb = new StringBuilder();
        switch (i) {
            case 0:
                if (i2 == 0) {
                    sb.append("F");
                    break;
                } else {
                    sb.append("A");
                    break;
                }
            case 1:
                sb.append("B");
                break;
            default:
                sb.append("D");
                break;
        }
        return sb.toString();
    }

    public static String innerLoopBreak(int i, int i2) {
        StringBuilder sb = new StringBuilder();
        switch (i) {
            case 0:
                for (int i3 = 0; i3 < 2; i3++) {
                    sb.append("L");
                    if (i2 != 0) {
                        sb.append("O");
                    }
                    break;
                }
                sb.append("O");
            case 1:
                sb.append("N");
                break;
            default:
                sb.append("D");
                break;
        }
        return sb.toString();
    }

    public static String innerSwitchBreak(int i, int i2) {
        StringBuilder sb = new StringBuilder();
        switch (i) {
            case 0:
                switch (i2) {
                    case 0:
                        sb.append("I");
                        break;
                    default:
                        sb.append("J");
                        break;
                }
                sb.append("O");
            case 1:
                sb.append("N");
                break;
            default:
                sb.append("D");
                break;
        }
        return sb.toString();
    }

    public static String terminalCase(int i) {
        StringBuilder sb = new StringBuilder();
        switch (i) {
            case 0:
                sb.append("R");
                return sb.toString();
            case 1:
                sb.append("T");
                throw new IllegalArgumentException(sb.toString());
            case 2:
                sb.append("C");
                break;
            default:
                sb.append("D");
                break;
        }
        return sb.toString();
    }

    public static String caughtExceptionThenFallthrough(int i, int i2) {
        StringBuilder sb = new StringBuilder();
        switch (i) {
            case 0:
                try {
                    sb.append(Integer.parseInt(i2 == 0 ? "7" : "x"));
                    break;
                } catch (NumberFormatException e) {
                    sb.append("C");
                    break;
                }
            case 1:
                sb.append("N");
                break;
            default:
                sb.append("D");
                break;
        }
        return sb.toString();
    }
}

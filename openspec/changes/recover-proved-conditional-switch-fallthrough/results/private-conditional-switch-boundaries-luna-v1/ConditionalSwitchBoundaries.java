public final class ConditionalSwitchBoundaries {
    private ConditionalSwitchBoundaries() {}

    public static String partialBreak(int selector, int takeBreak) {
        StringBuilder trace = new StringBuilder();
        switch (selector) {
            case 0:
                if (takeBreak != 0) {
                    trace.append("A");
                    break;
                }
                trace.append("F");
            case 1:
                trace.append("B");
                break;
            default:
                trace.append("D");
        }
        return trace.toString();
    }

    public static String innerLoopBreak(int selector, int breakLoop) {
        StringBuilder trace = new StringBuilder();
        switch (selector) {
            case 0:
                while (breakLoop != 0) {
                    trace.append("L");
                    break;
                }
                trace.append("O");
            case 1:
                trace.append("N");
                break;
            default:
                trace.append("D");
        }
        return trace.toString();
    }

    public static String innerSwitchBreak(int selector, int innerSelector) {
        StringBuilder trace = new StringBuilder();
        switch (selector) {
            case 0:
                switch (innerSelector) {
                    case 0:
                        trace.append("I");
                        break;
                    default:
                        trace.append("J");
                        break;
                }
                trace.append("O");
            case 1:
                trace.append("N");
                break;
            default:
                trace.append("D");
        }
        return trace.toString();
    }

    public static String terminalCase(int selector) {
        StringBuilder trace = new StringBuilder();
        switch (selector) {
            case 0:
                trace.append("R");
                return trace.toString();
            case 1:
                trace.append("T");
                throw new IllegalArgumentException(trace.toString());
            case 2:
                trace.append("C");
                break;
            default:
                trace.append("D");
        }
        return trace.toString();
    }

    public static String caughtExceptionThenFallthrough(int selector, int invalidInput) {
        StringBuilder trace = new StringBuilder();
        switch (selector) {
            case 0:
                try {
                    int value = Integer.parseInt(invalidInput == 0 ? "7" : "x");
                    trace.append(value);
                } catch (NumberFormatException exception) {
                    trace.append("C");
                }
            case 1:
                trace.append("N");
                break;
            default:
                trace.append("D");
        }
        return trace.toString();
    }
}

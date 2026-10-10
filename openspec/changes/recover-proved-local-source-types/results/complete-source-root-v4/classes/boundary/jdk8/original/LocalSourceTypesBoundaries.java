/** Small Java 8 source whose bytecode provides neighboring local-type proof cases. */
public final class LocalSourceTypesBoundaries {
    private static final String CHAR_INPUT = "./?A";
    private static char MUTABLE_CHAR_FIELD = 'F';

    private LocalSourceTypesBoundaries() {
    }

    // Positive char producer groups: exact C call, exact C field, i2c, and entry C parameter.
    // The call-seeded local also receives the only two legal int literal assignments to char.
    public static String charCallAndLiteralWrites(int index, boolean zero, boolean max) {
        char value = CHAR_INPUT.charAt(index);
        if (zero) {
            value = 0;
        }
        if (max) {
            value = 65535;
        }

        StringBuilder text = new StringBuilder();
        text.append(value);
        String label;
        switch (value) {
            case '.':
                label = "dot";
                break;
            case '/':
                label = "slash";
                break;
            case '?':
                label = "question";
                break;
            case 0:
                label = "zero";
                break;
            default:
                label = "other";
                break;
        }
        return label + ":" + (int) value + ":" + text.length();
    }

    public static String charFieldSeed() {
        char value = MUTABLE_CHAR_FIELD;
        StringBuilder text = new StringBuilder();
        text.append(value);
        return (int) value + ":" + text.length();
    }

    public static int charI2cSeed(int raw) {
        char value = (char) raw;
        StringBuilder text = new StringBuilder();
        text.append(value);
        return (int) value + text.length();
    }

    public static String charEntryParameterSeed(char seed, boolean max) {
        char value = seed;
        if (max) {
            value = 65535;
        }
        StringBuilder text = new StringBuilder();
        text.append(value);
        return (int) value + ":" + text.length();
    }

    // Negative char proof neighbors: a charAt producer is mixed with out-of-range literals,
    // ordinary arithmetic, or an input copy/merge. The local remains int and returns its value.
    public static int intWithOutOfRangeWrites(int selector, int input) {
        int value = CHAR_INPUT.charAt(0);
        if (selector == 0) {
            value = -1;
        } else if (selector == 1) {
            value = 65536;
        } else {
            value = input;
        }
        return value;
    }

    public static int intWithArithmeticWrite(boolean increment) {
        int value = CHAR_INPUT.charAt(0);
        if (increment) {
            value = value + 1;
        } else {
            value = 65535;
        }
        return value;
    }

    public static int intWithUnknownCopyMerge(boolean copy, int input) {
        int copied = input;
        int value = CHAR_INPUT.charAt(0);
        if (copy) {
            value = copied;
        } else {
            value = 65;
        }
        return value;
    }

    // Null-first positive: all later non-null writes have exact String type; no default means
    // selector 9 must print the original null value.
    public static void exactStringWritesAfterNull(int selector) {
        String value = null;
        switch (selector) {
            case 1:
                value = "one";
                break;
            case 2:
                value = new String("two");
                break;
            case 3:
                value = "three";
                break;
        }
        System.out.println(value);
    }

    // Negative reference neighborhoods: mixed exact reference types, all-null writes, and an
    // opaque Object-returning copy alongside a String write.
    public static void mixedReferenceWrites(boolean stringValue) {
        Object value = null;
        if (stringValue) {
            value = "text";
        } else {
            value = new StringBuilder("builder");
        }
        System.out.println(value);
    }

    public static void allNullWrites(boolean secondWrite) {
        Object value = null;
        if (secondWrite) {
            value = null;
        } else {
            value = null;
        }
        System.out.println(value);
    }

    private static Object opaqueCopy(Object input) {
        return input;
    }

    public static void unknownReferenceCopy(boolean copy, Object input) {
        Object value = null;
        if (copy) {
            value = opaqueCopy(input);
        } else {
            value = "known";
        }
        System.out.println(value);
    }

    // The two non-overlapping `value` locals are candidates for javac slot reuse. Their types
    // and lifetimes must be read from the resulting class/IR; this source does not assert a slot.
    public static String possibleSlotReuse(boolean stringValue) {
        String number;
        {
            int value = stringValue ? 65536 : -1;
            number = Integer.toString(value);
        }
        {
            String value = null;
            if (stringValue) {
                value = "reused-string";
            } else {
                value = new String("other-string");
            }
            return number + ":" + value;
        }
    }
}

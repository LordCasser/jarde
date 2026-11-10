public final class ReferenceCases {
    interface Answer {
        int value();
    }

    static final class FirstAnswer implements Answer {
        public int value() { return 31; }
    }

    static final class SecondAnswer implements Answer {
        public int value() { return 47; }
    }

    public static int interfaceCall(boolean chooseFirst) {
        Answer answer;
        if (chooseFirst) {
            answer = new FirstAnswer();
        } else {
            answer = new SecondAnswer();
        }
        return answer.value();
    }

    public static String nullCopyChain(boolean chooseFirst) {
        String result = null;
        String origin = null;
        if (chooseFirst) {
            origin = "north";
            result = origin;
        } else {
            origin = "south";
            result = origin;
        }
        String copy = result;
        String finalCopy = copy;
        return finalCopy;
    }

    public static char covariantArrayRead(boolean chooseStrings) {
        CharSequence[] values;
        if (chooseStrings) {
            values = new String[] { "alpha", "beta" };
        } else {
            values = new StringBuilder[] { new StringBuilder("gamma"), new StringBuilder("delta") };
        }
        return values[0].charAt(0);
    }

    public static Object objectExchange(boolean chooseFirst) {
        Object left;
        Object right;
        if (chooseFirst) {
            left = "left-value";
            right = Integer.valueOf(73);
            Object saved = left;
            left = right;
            right = saved;
            return left;
        } else {
            left = Long.valueOf(91L);
            right = "right-value";
            Object saved = right;
            right = left;
            left = saved;
            return left;
        }
    }

    public static String explicitCast(boolean succeeds) {
        Object value;
        if (succeeds) {
            value = "cast-ok";
        } else {
            value = Integer.valueOf(5);
        }
        return (String) value;
    }
}

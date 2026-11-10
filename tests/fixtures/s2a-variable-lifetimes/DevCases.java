public final class DevCases {
    private DevCases() {}

    public static int mergeSlots(boolean chooseLeft, int seed) {
        int merged;
        if (chooseLeft) {
            int leftOnly = seed + 17;
            merged = leftOnly * 2;
        } else {
            int rightOnly = seed - 9;
            merged = rightOnly * 3;
        }
        return merged;
    }

    public static int sequentialSlotLifetimes(int seed) {
        int total = 0;
        {
            int narrow = seed + 1;
            total += narrow;
        }
        {
            long wide = seed + 2L;
            total += (int) wide;
        }
        {
            double floating = seed + 3.5d;
            total += (int) floating;
        }
        {
            String reference = "" + seed;
            total += reference.length();
        }
        return total;
    }

    public static String branchLocalReferences(boolean first, int seed) {
        String selected;
        if (first) {
            String leftName = "left-" + seed;
            selected = leftName;
        } else {
            String rightName = "right-" + seed;
            selected = rightName;
        }
        return selected;
    }

    public static int swapAcrossBackedge(int left, int right, int rounds) {
        int checksum = 0;
        for (int i = 0; i < rounds; i++) {
            int temporary = left;
            left = right;
            right = temporary;
            checksum += left * 31 + right;
        }
        return checksum + left * 7 + right;
    }

    public static int criticalThenSibling(boolean takeEdge, boolean sibling, int value) {
        if (takeEdge) {
            value += 5;
        }
        if (sibling) {
            return value * 2;
        }
        return value - 3;
    }

    public static int handlerSeesPreThrowLocal(boolean shouldThrow, int input) {
        int observed = input + 2;
        try {
            if (shouldThrow) {
                observed = observed * 3;
                throw new Marker(7);
            }
            return observed;
        } catch (Marker caught) {
            return observed * 10 + caught.code;
        }
    }

    private static final class Marker extends RuntimeException {
        final int code;
        Marker(int code) { this.code = code; }
    }
}

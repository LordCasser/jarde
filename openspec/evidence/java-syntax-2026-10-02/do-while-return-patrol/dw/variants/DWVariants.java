public class DWVariants {
    // V1: throw shape — the do-while(false) body's abrupt edge is an athrow whose message reads
    // a local; the local-crossing closure refuses the method before the quote can matter.
    public static String throwInDoWhile() {
        int i = 0;
        while (i < 10) {
            i++;
            do {
                if (i % 3 == 0) break;
                if (i > 7) throw new IllegalStateException("early" + i);
            } while (false);
        }
        return "i=" + i;
    }
    // V4: the throw face of the silent miscompilation — the athrow leaf reads no local, so no
    // earlier closure fires; the quote it leaves holds the exceptional method exit.
    public static String throwConstDoWhile() {
        int i = 0;
        while (i < 10) {
            i++;
            do {
                if (i % 3 == 0) break;
                if (i > 7) throw new IllegalStateException("early");
            } while (false);
        }
        return "i=" + i;
    }

    // V2: double return — two conditional return leaves in one do-while(false) body.
    public static String doubleReturnDoWhile() {
        int i = 0;
        while (i < 10) {
            i++;
            do {
                if (i % 3 == 0) break;
                if (i > 7) return "a" + i;
                if (i > 8) return "b" + i;
            } while (false);
        }
        return "i=" + i;
    }
    // V3: the unrecoverable negative — the conditional return's leaf has two normal entries
    // (the short-circuit `||` spell), so no branch exclusively owns it and the body cannot own
    // the edge; the quote it leaves must close the whole method.
    public static String sharedLeafDoWhile() {
        int i = 0;
        while (i < 10) {
            i++;
            do {
                if (i % 3 == 0) break;
                if (i > 7 || i > 8) return "early" + i;
            } while (false);
        }
        return "i=" + i;
    }
    public static void main(String[] a) {
        try {
            System.out.println(throwInDoWhile());
        } catch (IllegalStateException e) {
            System.out.println(e.getMessage());
        }
        try {
            System.out.println(throwConstDoWhile());
        } catch (IllegalStateException e) {
            System.out.println(e.getMessage());
        }
        System.out.println(doubleReturnDoWhile());
        System.out.println(sharedLeafDoWhile());
    }
}

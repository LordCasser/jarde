public final class BranchReadNegatives {
    private BranchReadNegatives() {}

    static int loopCondition(int x) {
        boolean b = (x += 1) > 0 && x > 0;
        int n = 0;
        while (b) {
            n++;
            if (n > 2) {
                break;
            }
        }
        return n;
    }

    static int crossCatch(int x) {
        boolean b;
        try {
            throw null;
        } catch (NullPointerException caught) {
            b = (x += 1) > 0 && x > 0;
        }
        return b ? x : -1;
    }

    public static void main(String[] args) {
        System.out.println(loopCondition(0));
        System.out.println(loopCondition(-2));
        System.out.println(crossCatch(0));
    }
}

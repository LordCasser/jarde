public final class BranchReads {
    private BranchReads() {}

    static int ternaryRead(int x) {
        boolean b = (x += 1) > 0 && x > 0;
        return b ? x : -1;
    }

    static int ifStatement(int x) {
        boolean b = (x += 1) > 0 && x > 0;
        if (b) {
            return x;
        }
        return -1;
    }

    static boolean midChain(int x) {
        boolean b = (x += 1) > 0 && x > 0;
        return (x > 0) && b && (x < 100);
    }

    public static void main(String[] args) {
        System.out.println(ternaryRead(0));
        System.out.println(ternaryRead(-2));
        System.out.println(ifStatement(0));
        System.out.println(ifStatement(-2));
        System.out.println(midChain(0));
        System.out.println(midChain(-2));
    }
}

public class Runner {
    public static void main(String[] args) {
        if (BranchFinally.trace != 0) {
            throw new AssertionError("trace did not start at its default value");
        }
        int zero = BranchFinally.run(0);
        if (zero != 0 || BranchFinally.trace != 0) {
            throw new AssertionError("run(0): result=" + zero + " trace=" + BranchFinally.trace);
        }
        System.out.println("run(0)=" + zero + " trace=" + BranchFinally.trace);

        BranchFinally.trace = 0;
        int positive = BranchFinally.run(40);
        int expectedTrace = 1;
        for (int index = 1; index <= BranchFinally.DEPTH; index++) {
            expectedTrace += index;
        }
        if (positive != 40 || BranchFinally.trace != expectedTrace) {
            throw new AssertionError("run(40): result=" + positive + " trace=" + BranchFinally.trace
                    + " expectedTrace=" + expectedTrace);
        }
        System.out.println("run(40)=" + positive + " trace=" + BranchFinally.trace);
    }
}

public final class CatchAfterFieldAssignmentRunner {
    public static void main(String[] args) {
        for (boolean shouldThrow : new boolean[] { false, true }) {
            System.out.println("field=" + CatchAfterFieldAssignment.fieldAssignment(shouldThrow)
                    + ",local=" + CatchAfterFieldAssignment.localAssignment(shouldThrow));
        }
    }
}

public final class Runner {
    private Runner() {}

    public static void main(String[] args) {
        System.out.println(CompoundConditionNegative.prefixIncrement(false, 3, true, true));
        System.out.println(CompoundConditionNegative.prefixIncrement(true, 0, false, false));
        System.out.println(CompoundConditionNegative.prefixIncrement(true, -2, false, false));
        System.out.println(CompoundConditionNegative.prefixIncrement(true, -2, true, true));
        System.out.println(CompoundConditionNegative.localAssignment(false, 3, true, true));
        System.out.println(CompoundConditionNegative.localAssignment(true, 0, false, false));
        System.out.println(CompoundConditionNegative.localAssignment(true, -2, false, false));
        System.out.println(CompoundConditionNegative.localAssignment(true, -2, true, true));
    }
}

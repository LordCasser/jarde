package cf11;

/** Java 8 source probes for narrowly bounded forward conditional stack carries. */
public final class ForwardCarryProbe {
    public static int effects;
    public static int mutationCalls;
    public static int producerCalls;
    public static int armCalls;
    public static int otherCalls;
    public static int conditionCalls;
    public static String trace;

    private ForwardCarryProbe() {}

    public static void reset() {
        effects = 0;
        mutationCalls = 0;
        producerCalls = 0;
        armCalls = 0;
        otherCalls = 0;
        conditionCalls = 0;
        trace = "";
    }

    public static int mutate() {
        mutationCalls++;
        trace += "M";
        effects = 100;
        return 1;
    }

    /** Compound assignment must commit the saved left value only after the RHS returns. */
    public static void compoundCarry(boolean flag) {
        effects += flag ? mutate() : 1;
    }

    public static int lengthArm() {
        armCalls++;
        trace += "A";
        return 1;
    }

    /** Null length evaluation is physically before the conditional arm. */
    public static int lengthBeforeConditional(int[] values, boolean flag) {
        return values.length + (flag ? lengthArm() : 1);
    }

    public static int get() {
        producerCalls++;
        trace += "G";
        return 5;
    }

    public static int armEffect() {
        armCalls++;
        trace += "A";
        return 2;
    }

    public static int plainOther() {
        otherCalls++;
        trace += "B";
        return 3;
    }

    /** Direct local flag form: the producer is evaluated before the conditional expression. */
    public static int producerPlusFlag(boolean flag) {
        return get() + (flag ? armEffect() : plainOther());
    }

    public static boolean testFlag(boolean flag) {
        conditionCalls++;
        trace += "T";
        return flag;
    }

    /** Observable test call makes the producer/test/arm order visible in Runner output. */
    public static int producerPlusObservedTest(boolean flag) {
        return get() + (testFlag(flag) ? armEffect() : plainOther());
    }

    public static int mutateThenThrowRuntime() {
        mutationCalls++;
        trace += "R";
        effects = 100;
        throw new IllegalStateException("runtime-probe");
    }

    public static int mutateThenThrowError() {
        mutationCalls++;
        trace += "E";
        effects = 200;
        throw new AssertionError("error-probe");
    }

    /** On a throwing RHS, the compound assignment's final write must not happen. */
    public static void compoundRuntimeRhs(boolean flag) {
        effects += flag ? mutateThenThrowRuntime() : 1;
    }

    public static void compoundErrorRhs(boolean flag) {
        effects += flag ? mutateThenThrowError() : 1;
    }
}

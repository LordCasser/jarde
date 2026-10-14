package cf11;

/** Prints observations only; expected output is captured from the original JDK runs. */
public final class Runner {
    private Runner() {}

    private static void state(String label, String outcome, String value) {
        System.out.println(label + " outcome=" + outcome + " value=" + value
                + " effects=" + ForwardCarryProbe.effects
                + " mutationCalls=" + ForwardCarryProbe.mutationCalls
                + " producerCalls=" + ForwardCarryProbe.producerCalls
                + " armCalls=" + ForwardCarryProbe.armCalls
                + " otherCalls=" + ForwardCarryProbe.otherCalls
                + " conditionCalls=" + ForwardCarryProbe.conditionCalls
                + " trace=" + ForwardCarryProbe.trace);
    }

    private static void compound(boolean flag, String label) {
        ForwardCarryProbe.reset();
        try {
            ForwardCarryProbe.effects = 10;
            ForwardCarryProbe.compoundCarry(flag);
            state(label, "return", Integer.toString(ForwardCarryProbe.effects));
        } catch (RuntimeException failure) {
            state(label, failure.getClass().getName(), "not-returned");
        } catch (Error failure) {
            state(label, failure.getClass().getName(), "not-returned");
        }
    }

    private static void length(int[] values, boolean flag, String label) {
        ForwardCarryProbe.reset();
        try {
            int value = ForwardCarryProbe.lengthBeforeConditional(values, flag);
            state(label, "return", Integer.toString(value));
        } catch (RuntimeException failure) {
            state(label, failure.getClass().getName(), "not-returned");
        } catch (Error failure) {
            state(label, failure.getClass().getName(), "not-returned");
        }
    }

    private static void producer(boolean flag, boolean observedTest, String label) {
        ForwardCarryProbe.reset();
        try {
            int value = observedTest
                    ? ForwardCarryProbe.producerPlusObservedTest(flag)
                    : ForwardCarryProbe.producerPlusFlag(flag);
            state(label, "return", Integer.toString(value));
        } catch (RuntimeException failure) {
            state(label, failure.getClass().getName(), "not-returned");
        } catch (Error failure) {
            state(label, failure.getClass().getName(), "not-returned");
        }
    }

    private static void throwingRhs(boolean error, boolean flag, String label) {
        ForwardCarryProbe.reset();
        ForwardCarryProbe.effects = 10;
        try {
            if (error) {
                ForwardCarryProbe.compoundErrorRhs(flag);
            } else {
                ForwardCarryProbe.compoundRuntimeRhs(flag);
            }
            state(label, "return", Integer.toString(ForwardCarryProbe.effects));
        } catch (RuntimeException failure) {
            state(label, failure.getClass().getName(), "not-returned");
        } catch (Error failure) {
            state(label, failure.getClass().getName(), "not-returned");
        }
    }

    public static void main(String[] args) {
        compound(true, "compound-selected");
        compound(false, "compound-unselected");
        length(new int[] { 7, 8, 9 }, true, "length-nonnull-selected");
        length(null, true, "length-null-before-branch");
        producer(true, false, "producer-direct-selected");
        producer(false, false, "producer-direct-unselected");
        producer(true, true, "producer-observed-selected");
        producer(false, true, "producer-observed-unselected");
        throwingRhs(false, true, "rhs-runtime-throws");
        throwingRhs(true, true, "rhs-error-throws");
        throwingRhs(false, false, "rhs-runtime-control-return");
        throwingRhs(true, false, "rhs-error-control-return");
    }
}

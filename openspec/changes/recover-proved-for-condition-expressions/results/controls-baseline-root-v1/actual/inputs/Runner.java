package cf11;

public class Runner {
    private static void reportEffect(String label, String result, String exception) {
        System.out.println(label + " result=" + result + " exception=" + exception
                + " events=" + ForConditionEffects.events
                + " offsetCalls=" + ForConditionEffects.offsetCalls
                + " limitCalls=" + ForConditionEffects.limitCalls
                + " dynamicLimit=" + ForConditionEffects.dynamicLimit);
    }

    private static void reportRead(String label, String result, String exception) {
        System.out.println(label + " result=" + result + " exception=" + exception
                + " fieldBound=" + ForConditionReads.fieldBound
                + " negativeField=" + ForConditionReads.negativeField);
    }

    private static void reportRefusal(String label, String result, String exception) {
        System.out.println(label + " result=" + result + " exception=" + exception
                + " counter=" + ForConditionRefusals.counter
                + " limitCalls=" + ForConditionRefusals.limitCalls);
    }

    public static void main(String[] args) {
        ForConditionEffects.events = 0;
        ForConditionEffects.offsetCalls = 0;
        ForConditionEffects.limitCalls = 0;
        ForConditionEffects.offsetValue = 0;
        ForConditionEffects.limitValue = 0;
        ForConditionEffects.offsetThrowAt = 0;
        ForConditionEffects.limitThrowAt = 0;
        ForConditionEffects.dynamicLimit = 0;
        try {
            reportEffect("functions-zero", Integer.toString(ForConditionEffects.sumWithFunctionBounds()), "none");
        } catch (RuntimeException error) {
            reportEffect("functions-zero", "-", error.getClass().getName());
        }

        ForConditionEffects.events = 0;
        ForConditionEffects.offsetCalls = 0;
        ForConditionEffects.limitCalls = 0;
        ForConditionEffects.offsetValue = 0;
        ForConditionEffects.limitValue = 2;
        ForConditionEffects.offsetThrowAt = 0;
        ForConditionEffects.limitThrowAt = 0;
        try {
            reportEffect("functions-two", Integer.toString(ForConditionEffects.sumWithFunctionBounds()), "none");
        } catch (RuntimeException error) {
            reportEffect("functions-two", "-", error.getClass().getName());
        }

        ForConditionEffects.events = 0;
        ForConditionEffects.offsetCalls = 0;
        ForConditionEffects.limitCalls = 0;
        ForConditionEffects.offsetValue = 0;
        ForConditionEffects.limitValue = 2;
        ForConditionEffects.offsetThrowAt = 1;
        ForConditionEffects.limitThrowAt = 0;
        try {
            reportEffect("offset-first", Integer.toString(ForConditionEffects.sumWithFunctionBounds()), "none");
        } catch (RuntimeException error) {
            reportEffect("offset-first", "-", error.getClass().getName());
        }

        ForConditionEffects.events = 0;
        ForConditionEffects.offsetCalls = 0;
        ForConditionEffects.limitCalls = 0;
        ForConditionEffects.offsetValue = 0;
        ForConditionEffects.limitValue = 2;
        ForConditionEffects.offsetThrowAt = 2;
        ForConditionEffects.limitThrowAt = 0;
        try {
            reportEffect("offset-later", Integer.toString(ForConditionEffects.sumWithFunctionBounds()), "none");
        } catch (RuntimeException error) {
            reportEffect("offset-later", "-", error.getClass().getName());
        }

        ForConditionEffects.events = 0;
        ForConditionEffects.offsetCalls = 0;
        ForConditionEffects.limitCalls = 0;
        ForConditionEffects.offsetValue = 0;
        ForConditionEffects.limitValue = 2;
        ForConditionEffects.offsetThrowAt = 0;
        ForConditionEffects.limitThrowAt = 1;
        try {
            reportEffect("limit-first", Integer.toString(ForConditionEffects.sumWithFunctionBounds()), "none");
        } catch (RuntimeException error) {
            reportEffect("limit-first", "-", error.getClass().getName());
        }

        ForConditionEffects.events = 0;
        ForConditionEffects.offsetCalls = 0;
        ForConditionEffects.limitCalls = 0;
        ForConditionEffects.offsetValue = 0;
        ForConditionEffects.limitValue = 2;
        ForConditionEffects.offsetThrowAt = 0;
        ForConditionEffects.limitThrowAt = 2;
        try {
            reportEffect("limit-later", Integer.toString(ForConditionEffects.sumWithFunctionBounds()), "none");
        } catch (RuntimeException error) {
            reportEffect("limit-later", "-", error.getClass().getName());
        }

        ForConditionEffects.events = 0;
        ForConditionEffects.offsetCalls = 0;
        ForConditionEffects.limitCalls = 0;
        ForConditionEffects.offsetValue = 0;
        ForConditionEffects.limitValue = 0;
        ForConditionEffects.offsetThrowAt = 0;
        ForConditionEffects.limitThrowAt = 0;
        ForConditionEffects.dynamicLimit = 3;
        try {
            reportEffect("dynamic-bound", Integer.toString(ForConditionEffects.sumWithChangingFieldBound()), "none");
        } catch (RuntimeException error) {
            reportEffect("dynamic-bound", "-", error.getClass().getName());
        }

        ForConditionEffects.events = 0;
        ForConditionEffects.offsetCalls = 0;
        ForConditionEffects.limitCalls = 0;
        ForConditionEffects.offsetValue = 0;
        ForConditionEffects.limitValue = 2;
        ForConditionEffects.offsetThrowAt = 0;
        ForConditionEffects.limitThrowAt = 0;
        ForConditionEffects.dynamicLimit = 0;
        try {
            reportEffect("functions-two-continue", Integer.toString(ForConditionEffects.sumWithFunctionBoundsAndContinue()), "none");
        } catch (RuntimeException error) {
            reportEffect("functions-two-continue", "-", error.getClass().getName());
        }

        ForConditionRefusals.counter = 0;
        ForConditionRefusals.limitCalls = 0;
        ForConditionRefusals.limit = 2;
        try {
            reportRefusal("while-true-tick-break", Integer.toString(ForConditionRefusals.sumWithLeadingTickAndBreak()), "none");
        } catch (RuntimeException error) {
            reportRefusal("while-true-tick-break", "-", error.getClass().getName());
        }

        ForConditionRefusals.counter = 0;
        ForConditionRefusals.limitCalls = 0;
        ForConditionRefusals.limit = 2;
        try {
            reportRefusal("while-assigned-call-bound", Integer.toString(ForConditionRefusals.sumWithAssignedCallBound()), "none");
        } catch (RuntimeException error) {
            reportRefusal("while-assigned-call-bound", "-", error.getClass().getName());
        }

        ForConditionReads.fieldBound = 3;
        ForConditionReads.negativeField = 0;
        try {
            reportRead("field-bound", Integer.toString(ForConditionReads.sumWithChangingFieldBound()), "none");
        } catch (RuntimeException error) {
            reportRead("field-bound", "-", error.getClass().getName());
        }

        ForConditionReads.fieldBound = 0;
        ForConditionReads.negativeField = 0;
        try {
            reportRead("array-length", Integer.toString(ForConditionReads.sumWithArrayLength(new int[] {4, 5})), "none");
        } catch (RuntimeException error) {
            reportRead("array-length", "-", error.getClass().getName());
        }

        ForConditionReads.fieldBound = 0;
        ForConditionReads.negativeField = 0;
        try {
            reportRead("array-length-null", Integer.toString(ForConditionReads.sumWithArrayLength(null)), "none");
        } catch (RuntimeException error) {
            reportRead("array-length-null", "-", error.getClass().getName());
        }

        ForConditionReads.fieldBound = 0;
        ForConditionReads.negativeField = 0;
        try {
            reportRead("array-element", Integer.toString(ForConditionReads.countWithArrayElement(new int[] {2, 0})), "none");
        } catch (RuntimeException error) {
            reportRead("array-element", "-", error.getClass().getName());
        }

        ForConditionReads.fieldBound = 0;
        ForConditionReads.negativeField = 0;
        try {
            reportRead("array-element-bounds", Integer.toString(ForConditionReads.countWithArrayElement(new int[] {3})), "none");
        } catch (RuntimeException error) {
            reportRead("array-element-bounds", "-", error.getClass().getName());
        }

        ForConditionReads.fieldBound = 0;
        ForConditionReads.negativeField = 0;
        try {
            reportRead("array-element-null", Integer.toString(ForConditionReads.countWithArrayElement(null)), "none");
        } catch (RuntimeException error) {
            reportRead("array-element-null", "-", error.getClass().getName());
        }

        ForConditionReads.fieldBound = 0;
        ForConditionReads.negativeField = -3;
        try {
            reportRead("negated-field", Integer.toString(ForConditionReads.sumWithNegatedField()), "none");
        } catch (RuntimeException error) {
            reportRead("negated-field", "-", error.getClass().getName());
        }

        ForConditionReads.fieldBound = 0;
        ForConditionReads.negativeField = 0;
        try {
            reportRead("negated-parameter", Integer.toString(ForConditionReads.sumWithNegatedParameter(-3)), "none");
        } catch (RuntimeException error) {
            reportRead("negated-parameter", "-", error.getClass().getName());
        }

        ForConditionReads.fieldBound = 0;
        ForConditionReads.negativeField = 0;
        try {
            reportRead("written-local-while", Integer.toString(ForConditionReads.oldWhileWithWrittenLocalBound()), "none");
        } catch (RuntimeException error) {
            reportRead("written-local-while", "-", error.getClass().getName());
        }
    }
}

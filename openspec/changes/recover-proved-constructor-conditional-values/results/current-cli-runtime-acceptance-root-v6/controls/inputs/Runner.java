package cf11;

public final class Runner {
    private Runner() { }

    private static String value(String text) {
        return text == null ? "null" : text + ":" + text.length();
    }

    private static void positive(String label, String result, String first, String second) {
        System.out.println(label + "|value=" + value(result)
                + "|sameFirst=" + (result == first)
                + "|sameSecond=" + (result == second)
                + "|events=" + ConditionalConstructionPositive.events
                + "|conditions=" + ConditionalConstructionPositive.conditionCalls
                + "|arguments=" + ConditionalConstructionPositive.argumentCalls);
    }

    private static void positiveError(String label, Throwable error) {
        System.out.println(label + "|error=" + error.getClass().getName()
                + "|events=" + ConditionalConstructionPositive.events
                + "|conditions=" + ConditionalConstructionPositive.conditionCalls
                + "|arguments=" + ConditionalConstructionPositive.argumentCalls);
    }

    private static void refusal(String label, String result, String first, String second) {
        System.out.println(label + "|value=" + value(result)
                + "|sameFirst=" + (result == first)
                + "|sameSecond=" + (result == second)
                + "|sameSharedSlot=" + (result == ConditionalConstructionRefusals.sharedSlot)
                + "|events=" + ConditionalConstructionRefusals.events
                + "|conditions=" + ConditionalConstructionRefusals.conditionCalls
                + "|arguments=" + ConditionalConstructionRefusals.argumentCalls
                + "|effects=" + ConditionalConstructionRefusals.effectCalls);
    }

    private static void refusalError(String label, Throwable error) {
        System.out.println(label + "|error=" + error.getClass().getName()
                + "|events=" + ConditionalConstructionRefusals.events
                + "|conditions=" + ConditionalConstructionRefusals.conditionCalls
                + "|arguments=" + ConditionalConstructionRefusals.argumentCalls
                + "|effects=" + ConditionalConstructionRefusals.effectCalls);
    }

    public static void main(String[] args) {
        String original = new String("seed");
        ConditionalConstructionPositive.reset();
        try {
            String result = ConditionalConstructionPositive.oneConstructedArm(true, false, false, original);
            positive("one-selected-new", result, original, null);
        } catch (Throwable error) {
            positiveError("one-selected-new", error);
        }

        original = new String("seed");
        ConditionalConstructionPositive.reset();
        try {
            String result = ConditionalConstructionPositive.oneConstructedArm(false, false, true, original);
            positive("one-reuse-skips-throwing-argument", result, original, null);
        } catch (Throwable error) {
            positiveError("one-reuse-skips-throwing-argument", error);
        }

        ConditionalConstructionPositive.reset();
        try {
            String result = ConditionalConstructionPositive.oneConstructedArm(true, false, false, null);
            positive("one-selected-null-constructor-throws", result, null, null);
        } catch (Throwable error) {
            positiveError("one-selected-null-constructor-throws", error);
        }

        ConditionalConstructionPositive.reset();
        try {
            String result = ConditionalConstructionPositive.oneConstructedArm(false, false, false, null);
            positive("one-unselected-null-reuse", result, null, null);
        } catch (Throwable error) {
            positiveError("one-unselected-null-reuse", error);
        }

        original = new String("seed");
        ConditionalConstructionPositive.reset();
        try {
            String result = ConditionalConstructionPositive.oneConstructedArm(true, false, true, original);
            positive("one-selected-argument-throws", result, original, null);
        } catch (Throwable error) {
            positiveError("one-selected-argument-throws", error);
        }

        original = new String("seed");
        ConditionalConstructionPositive.reset();
        try {
            String result = ConditionalConstructionPositive.oneConstructedArm(true, true, false, original);
            positive("condition-throws-before-argument", result, original, null);
        } catch (Throwable error) {
            positiveError("condition-throws-before-argument", error);
        }

        String left = new String("first");
        String right = new String("second");
        ConditionalConstructionPositive.reset();
        try {
            String result = ConditionalConstructionPositive.twoConstructedArms(true, false, false, true, left, right);
            positive("two-arm-first-only", result, left, right);
        } catch (Throwable error) {
            positiveError("two-arm-first-only", error);
        }

        left = new String("first");
        right = new String("second");
        ConditionalConstructionPositive.reset();
        try {
            String result = ConditionalConstructionPositive.twoConstructedArms(false, false, true, false, left, right);
            positive("two-arm-second-only", result, left, right);
        } catch (Throwable error) {
            positiveError("two-arm-second-only", error);
        }

        original = new String("slot");
        ConditionalConstructionPositive.reset();
        try {
            ConditionalConstructionPositive.writeStaticSlot(true, false, false, original);
            positive("static-field-consumer", ConditionalConstructionPositive.staticSlot, original, null);
        } catch (Throwable error) {
            positiveError("static-field-consumer", error);
        }

        original = new String("slot");
        ConditionalConstructionPositive.reset();
        ConditionalConstructionPositive instance = new ConditionalConstructionPositive();
        try {
            instance.writeInstanceSlot(true, false, false, original);
            positive("instance-field-consumer", instance.instanceSlot, original, null);
        } catch (Throwable error) {
            positiveError("instance-field-consumer", error);
        }

        original = new String("slot-reused");
        ConditionalConstructionPositive.reset();
        try {
            ConditionalConstructionPositive.writeStaticSlot(false, false, true, original);
            positive("static-field-reuses-original", ConditionalConstructionPositive.staticSlot, original, null);
        } catch (Throwable error) {
            positiveError("static-field-reuses-original", error);
        }

        original = new String("slot-reused");
        ConditionalConstructionPositive.reset();
        instance = new ConditionalConstructionPositive();
        try {
            instance.writeInstanceSlot(false, false, true, original);
            positive("instance-field-reuses-original", instance.instanceSlot, original, null);
        } catch (Throwable error) {
            positiveError("instance-field-reuses-original", error);
        }

        original = new String("slot-throw");
        ConditionalConstructionPositive.reset();
        try {
            ConditionalConstructionPositive.writeStaticSlot(true, false, true, original);
            positive("static-field-selected-argument-throws", ConditionalConstructionPositive.staticSlot, original, null);
        } catch (Throwable error) {
            System.out.println("static-field-selected-argument-throws|error=" + error.getClass().getName()
                    + "|slot=" + value(ConditionalConstructionPositive.staticSlot)
                    + "|slotIsNull=" + (ConditionalConstructionPositive.staticSlot == null)
                    + "|events=" + ConditionalConstructionPositive.events
                    + "|conditions=" + ConditionalConstructionPositive.conditionCalls
                    + "|arguments=" + ConditionalConstructionPositive.argumentCalls);
        }

        ConditionalConstructionRefusals.reset();
        String originalRefusal = new String("reuse");
        try {
            String result = ConditionalConstructionRefusals.localBoundary(true, new String("local"), originalRefusal);
            refusal("local-boundary", result, originalRefusal, null);
        } catch (Throwable error) {
            refusalError("local-boundary", error);
        }

        ConditionalConstructionRefusals.reset();
        originalRefusal = new String("reuse");
        try {
            String result = ConditionalConstructionRefusals.sharedFieldAndReturn(true, new String("shared"), originalRefusal);
            refusal("shared-field-and-return", result, originalRefusal, null);
        } catch (Throwable error) {
            refusalError("shared-field-and-return", error);
        }

        ConditionalConstructionRefusals.reset();
        originalRefusal = new String("inner");
        try {
            String result = ConditionalConstructionRefusals.nestedConditionals(true, true,
                    new String("inner"), originalRefusal, new String("outer"));
            refusal("nested-inner-selected", result, originalRefusal, null);
        } catch (Throwable error) {
            refusalError("nested-inner-selected", error);
        }

        ConditionalConstructionRefusals.reset();
        originalRefusal = new String("fallback");
        try {
            String result = ConditionalConstructionRefusals.independentBranchEffect(true,
                    new String("effect"), originalRefusal);
            refusal("independent-branch-effect", result, originalRefusal, null);
        } catch (Throwable error) {
            refusalError("independent-branch-effect", error);
        }

        ConditionalConstructionRefusals.reset();
        originalRefusal = new String("fallback");
        try {
            String result = ConditionalConstructionRefusals.handlerAroundConditional(true, true,
                    new String("throw"), originalRefusal);
            refusal("handler-selected-throw-caught", result, originalRefusal, null);
        } catch (Throwable error) {
            refusalError("handler-selected-throw-caught", error);
        }

        ConditionalConstructionRefusals.reset();
        originalRefusal = new String("fallback");
        try {
            String result = ConditionalConstructionRefusals.handlerAroundConditional(false, true,
                    new String("throw"), originalRefusal);
            refusal("handler-unselected-throw-skipped", result, originalRefusal, null);
        } catch (Throwable error) {
            refusalError("handler-unselected-throw-skipped", error);
        }
    }
}

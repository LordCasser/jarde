public class ShiftEffectOrderRunner {
    public static void main(String[] args) {
        for (int value : new int[] { -7, 0, 7 }) {
            System.out.println(value + ":" + ShiftEffectOrder.inline(value) + ":"
                    + ShiftEffectOrder.separated(value) + ":"
                    + throwLeft(value) + ":" + throwRight(value));
        }
    }

    private static int throwLeft(int value) {
        try {
            ShiftEffectOrder.throwLeft(value);
            throw new AssertionError("expected left operand to throw");
        } catch (IllegalArgumentException expected) {
            return ShiftEffectOrder.events();
        }
    }

    private static int throwRight(int value) {
        try {
            ShiftEffectOrder.throwRight(value);
            throw new AssertionError("expected right operand to throw");
        } catch (IllegalArgumentException expected) {
            return ShiftEffectOrder.events();
        }
    }
}

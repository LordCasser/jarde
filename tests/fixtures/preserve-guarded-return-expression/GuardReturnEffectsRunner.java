public final class GuardReturnEffectsRunner {
    public static void main(String[] args) {
        GuardReturnEffects value = new GuardReturnEffects();
        value.n = 5;
        GuardReturnEffects.calls = 0;
        System.out.println("effect:" + value.effect(false) + ":" + value.n + ":" + GuardReturnEffects.calls);

        value.n = 7;
        GuardReturnEffects.calls = 0;
        try {
            value.effect(true);
            System.out.println("effect-throw:missing");
        } catch (IllegalStateException failure) {
            System.out.println("effect-throw:" + failure.getMessage() + ":" + value.n + ":" + GuardReturnEffects.calls);
        }
        synchronized (value) {
            System.out.println("effect-lock:released");
        }

        value.n = 3;
        GuardReturnEffects.calls = 0;
        System.out.println("nested:" + value.nested(false) + ":" + value.n + ":" + GuardReturnEffects.calls);
        value.n = 4;
        GuardReturnEffects.calls = 0;
        try {
            value.nested(true);
            System.out.println("nested-throw:missing");
        } catch (IllegalStateException failure) {
            System.out.println("nested-throw:" + failure.getMessage() + ":" + value.n + ":" + GuardReturnEffects.calls);
        }
        synchronized (value) {
            System.out.println("nested-lock:released");
        }
    }
}

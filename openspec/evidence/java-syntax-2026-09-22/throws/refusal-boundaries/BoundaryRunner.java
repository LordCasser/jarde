public final class BoundaryRunner {
    public static void main(String[] args) {
        if (args.length != 1) {
            throw new IllegalArgumentException("mode required");
        }
        switch (args[0]) {
            case "parameter":
                parameter();
                break;
            case "call":
                call();
                break;
            case "lower":
                lower();
                break;
            default:
                throw new IllegalArgumentException(args[0]);
        }
    }

    private static void parameter() {
        RuntimeException expected = new IllegalArgumentException("parameter");
        try {
            ThrowProbe.parameter(expected);
            System.out.println("parameter=returned");
        } catch (Throwable actual) {
            System.out.println("parameter=" + actual.getClass().getName() + ":" +
                (actual == expected) + ":" + ThrowEffects.calls);
        }
    }

    private static void call() {
        RuntimeException expected = new IllegalArgumentException("call");
        ThrowEffects.expected = expected;
        ThrowEffects.failure = null;
        ThrowEffects.calls = 0;
        try {
            ThrowProbe.call();
            System.out.println("call=returned:" + ThrowEffects.calls);
        } catch (Throwable actual) {
            System.out.println("call=" + actual.getClass().getName() + ":" +
                (actual == expected) + ":" + ThrowEffects.calls);
        }
    }

    private static void lower() {
        RuntimeException parameter = new IllegalArgumentException("parameter");
        RuntimeException producer = new IllegalStateException("producer");
        ThrowEffects.expected = producer;
        ThrowEffects.failure = null;
        ThrowEffects.calls = 0;
        try {
            ThrowProbe.parameter(parameter);
            System.out.println("lower=returned:" + ThrowEffects.calls);
        } catch (Throwable actual) {
            System.out.println("lower=" + actual.getClass().getName() + ":" +
                (actual == parameter) + ":producer=" + (actual == producer) + ":" +
                ThrowEffects.calls);
        }
    }
}

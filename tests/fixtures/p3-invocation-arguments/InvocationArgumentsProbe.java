public class InvocationArgumentsProbe {
    public static int objectString() {
        return ArgumentTarget.object((Object) "x");
    }

    public static int objectNull() {
        return ArgumentTarget.object((Object) null);
    }

    public static int objectArray(String[] value) {
        return ArgumentTarget.array((Object) value);
    }

    public static int objectBoxed() {
        return ArgumentTarget.boxed((Object) Integer.valueOf(3));
    }

    public static int widening(char value) {
        return ArgumentTarget.number((int) value);
    }

    public static int narrowByte() {
        return ArgumentTarget.narrow((byte) 1);
    }

    public static int narrowShort() {
        return ArgumentTarget.narrow((short) 1);
    }

    public static int constructorObject() {
        return new ArgumentTarget((Object) "x").value();
    }

    public static int multiple(char value) {
        return ArgumentTarget.multi((Object) "a", (Object) "b", (int) value);
    }

    public static int genericObject() {
        return ArgumentTarget.object((Object) GenericFactory.make());
    }

    public static int functionalRunnable() {
        return ArgumentTarget.functional((Runnable) InvocationArgumentsProbe::returnFive);
    }

    public static int functionalObject() {
        return ArgumentTarget.objectFunctional((Object) (Runnable) InvocationArgumentsProbe::returnFive);
    }

    private static int returnFive() {
        return 5;
    }
}

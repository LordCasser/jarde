import java.lang.reflect.Method;

public final class ReturnBoundaryRunner {
    private ReturnBoundaryRunner() {}

    public static void main(String[] args) throws Exception {
        Class<?> booleans = Class.forName("BooleanReturnBoundaries");
        for (String name : new String[] {"booleanAsByte", "booleanAsChar", "booleanAsShort"}) {
            Method method = booleans.getMethod(name, boolean.class);
            for (boolean value : new boolean[] {false, true}) {
                Object result = method.invoke(null, value);
                if (result instanceof Character) result = (int) (Character) result;
                System.out.println(name + ":" + value + ":" + result);
            }
        }
        Class<?> rawTwo = Class.forName("BooleanRawTwoCaller");
        for (String name : new String[] {"byteValue", "charValue", "shortValue"}) {
            System.out.println(name + ":" + rawTwo.getMethod(name).invoke(null));
        }
        Method asBoolean = booleans.getMethod("integerAsBoolean", int.class);
        for (int value : new int[] {0, 1, 2, 3, -1}) {
            System.out.println("integerAsBoolean:" + value + ":" + asBoolean.invoke(null, value));
        }
    }
}

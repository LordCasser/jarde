package em23;

import java.lang.reflect.Constructor;
import java.lang.reflect.Field;

public class Runner {
    private static InputFieldMultiplyControls freshTarget(int value) throws Exception {
        InputFieldMultiplyControls target = new InputFieldMultiplyControls();
        Class<?>[] nestedTypes = InputFieldMultiplyControls.class.getDeclaredClasses();
        if (nestedTypes.length != 1) {
            throw new AssertionError("expected exactly one nested receiver class");
        }
        Constructor<?> constructor = nestedTypes[0].getDeclaredConstructor();
        constructor.setAccessible(true);
        Object nested = constructor.newInstance();
        Field receiver = InputFieldMultiplyControls.class.getField("a");
        receiver.set(target, nested);
        Field field = nested.getClass().getDeclaredField("f");
        field.setAccessible(true);
        field.setInt(nested, value);
        return target;
    }

    private static int readF(InputFieldMultiplyControls target) throws Exception {
        Object nested = InputFieldMultiplyControls.class.getField("a").get(target);
        Field value = nested.getClass().getDeclaredField("f");
        value.setAccessible(true);
        return value.getInt(nested);
    }

    private static String thrown(Runnable action) {
        try {
            action.run();
            return "none";
        } catch (RuntimeException error) {
            return error.getClass().getSimpleName();
        }
    }

    private static void expect(String actual, String expected, String label) {
        if (!expected.equals(actual)) {
            throw new AssertionError(label + ": expected " + expected + ", got " + actual);
        }
    }

    public static void main(String[] args) throws Exception {
        InputFieldMultiplyControls explicit = freshTarget(5);
        explicit.test1(3);
        System.out.println("explicit-add=" + readF(explicit));

        InputFieldMultiplyControls normal = freshTarget(5);
        normal.test2(4);
        System.out.println("multiply-normal=" + readF(normal));

        InputFieldMultiplyControls overflow = freshTarget(Integer.MAX_VALUE);
        overflow.test2(3);
        System.out.println("multiply-max=" + readF(overflow));

        InputFieldMultiplyControls minOverflow = freshTarget(Integer.MIN_VALUE);
        minOverflow.test2(-1);
        System.out.println("multiply-min-neg1=" + readF(minOverflow));

        InputFieldMultiplyControls zero = freshTarget(7);
        zero.test2(0);
        System.out.println("multiply-zero=" + readF(zero));

        InputFieldMultiplyControls negative = freshTarget(5);
        negative.test2(-3);
        System.out.println("multiply-negative=" + readF(negative));

        InputFieldMultiplyControls nullMultiply = new InputFieldMultiplyControls();
        nullMultiply.a = null;
        String nullMultiplyFailure = thrown(() -> nullMultiply.test2(2));
        expect(nullMultiplyFailure, "NullPointerException", "test2 null receiver");
        System.out.println("multiply-null=" + nullMultiplyFailure);

        InputFieldMultiplyControls divide = freshTarget(3);
        int divided = divide.multiplyDivide(2);
        System.out.println("multiply-divide-normal=" + divided + ",field=" + readF(divide));

        InputFieldMultiplyControls divideZero = freshTarget(17);
        String divideZeroFailure = thrown(() -> divideZero.multiplyDivide(0));
        expect(divideZeroFailure, "ArithmeticException", "non-null divide by zero");
        System.out.println("multiply-divide-zero=" + divideZeroFailure + ",field=" + readF(divideZero));

        InputFieldMultiplyControls nullDivide = new InputFieldMultiplyControls();
        nullDivide.a = null;
        String nullDivideFailure = thrown(() -> nullDivide.multiplyDivide(0));
        expect(nullDivideFailure, "NullPointerException", "null receiver before divide");
        System.out.println("multiply-divide-null-zero=" + nullDivideFailure);
    }
}

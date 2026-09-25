import java.util.function.Supplier;

public class ArgumentTarget {
    private final int constructorCode;

    public ArgumentTarget(Object value) {
        constructorCode = 1;
    }

    public ArgumentTarget(String value) {
        constructorCode = 2;
    }

    public int value() {
        return constructorCode;
    }

    public static int object(Object value) {
        return 3;
    }

    public static int object(String value) {
        return 4;
    }

    public static int array(Object value) {
        return 5;
    }

    public static int array(String[] value) {
        return 6;
    }

    public static int boxed(Object value) {
        return 7;
    }

    public static int boxed(Integer value) {
        return 8;
    }

    public static int number(char value) {
        return 9;
    }

    public static int number(int value) {
        return 10;
    }

    public static int narrow(byte value) {
        return 11;
    }

    public static int narrow(short value) {
        return 12;
    }

    public static int narrow(int value) {
        return 13;
    }

    public static int multi(Object first, Object second, char third) {
        return 14;
    }

    public static int multi(Object first, Object second, int third) {
        return 15;
    }

    public static int functional(Runnable value) {
        return 16;
    }

    public static int functional(Supplier<Integer> value) {
        return 17;
    }

    public static int objectFunctional(Object value) {
        return 18;
    }

    public static int objectFunctional(Runnable value) {
        return 19;
    }
}

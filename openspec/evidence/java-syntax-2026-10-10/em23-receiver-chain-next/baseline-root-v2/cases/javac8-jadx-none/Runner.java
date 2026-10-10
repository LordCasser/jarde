package em23;

import java.lang.reflect.Constructor;
import java.lang.reflect.Field;

public class Runner {
    private static InputFieldIncrement2 freshTarget() throws Exception {
        InputFieldIncrement2 target = new InputFieldIncrement2();
        Class<?>[] nestedTypes = InputFieldIncrement2.class.getDeclaredClasses();
        if (nestedTypes.length != 1) {
            throw new AssertionError("expected exactly one nested receiver class");
        }
        Constructor<?> constructor = nestedTypes[0].getDeclaredConstructor();
        constructor.setAccessible(true);
        Object nested = constructor.newInstance();
        Field receiver = InputFieldIncrement2.class.getField("a");
        receiver.set(target, nested);
        return target;
    }

    private static int readF(InputFieldIncrement2 target) throws Exception {
        Object nested = InputFieldIncrement2.class.getField("a").get(target);
        Field value = nested.getClass().getDeclaredField("f");
        value.setAccessible(true);
        return value.getInt(nested);
    }

    public static void main(String[] args) throws Exception {
        InputFieldIncrement2 addTarget = freshTarget();
        addTarget.test1(3);
        System.out.println("add=" + readF(addTarget));

        InputFieldIncrement2 multiplyTarget = freshTarget();
        multiplyTarget.test2(4);
        System.out.println("multiply=" + readF(multiplyTarget));
    }
}

package dt29p3;

import java.lang.reflect.Field;
import java.lang.reflect.Method;

public class Runner {
    private static String bits(boolean... values) throws Exception {
        Class<?> aClass = Class.forName("dt29p3.BranchedBits$A");
        Class<?> bClass = Class.forName("dt29p3.BranchedBits$B");
        Object a = bClass.getConstructor().newInstance();
        String[] names = {"publicField", "protectedField", "packagePrivateField", "privateField"};
        for (int i = 0; i < names.length; i++) {
            Field field = aClass.getDeclaredField(names[i]);
            field.setAccessible(true);
            field.setBoolean(a, values[i]);
        }
        Method method = BranchedBits.class.getMethod("bits", aClass);
        return (String) method.invoke(null, a);
    }

    public static void main(String[] args) throws Exception {
        System.out.println(bits(true, true, true, true));
        System.out.println(bits(false, false, false, false));
        System.out.println(bits(true, false, true, false));
        System.out.println(bits(false, true, false, true));
    }
}

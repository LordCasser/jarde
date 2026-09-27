package dt29p3;

import java.lang.reflect.Field;
import java.lang.reflect.Method;

public class AlternateRunner {
    public static void main(String[] args) throws Exception {
        Class<?> aClass = Class.forName("dt29p3.BranchedBitsAlternate$A");
        Object a = aClass.getConstructor().newInstance();
        for (String name : new String[] {"alpha", "beta", "gamma", "delta"}) {
            Field field = aClass.getDeclaredField(name);
            field.setAccessible(true);
            field.setBoolean(a, true);
        }
        Method bits = BranchedBitsAlternate.class.getMethod("bits", aClass);
        String value = (String) bits.invoke(null, a);
        int reads = aClass.getField("reads").getInt(null);
        boolean alpha = aClass.getField("alpha").getBoolean(a);
        Field betaField = aClass.getDeclaredField("beta");
        betaField.setAccessible(true);
        boolean beta = betaField.getBoolean(a);
        Field gammaField = aClass.getDeclaredField("gamma");
        gammaField.setAccessible(true);
        boolean gamma = gammaField.getBoolean(a);
        System.out.println(value + ":" + reads + ":" + alpha + ":" + beta + ":" + gamma);
    }
}

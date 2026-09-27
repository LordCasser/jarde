package dt16;

import java.lang.reflect.Method;

public class Runner {
    public static void main(String[] args) throws Exception {
        NullResult result = new NullResult();
        Integer value = result.<Integer>value();
        Method method = NullResult.class.getDeclaredMethod("value");
        System.out.println((value == null) + ":" + method.getTypeParameters()[0].getName()
                + ":" + method.getTypeParameters()[0].getBounds()[0].getTypeName()
                + ":" + method.getGenericReturnType().getTypeName());
    }
}

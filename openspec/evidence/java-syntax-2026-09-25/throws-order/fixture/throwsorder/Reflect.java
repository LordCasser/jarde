package throwsorder;

import java.lang.reflect.Method;

public class Reflect {
    public static void main(String[] names) throws Exception {
        for (String name : names) {
            Method method = Class.forName("throwsorder." + name).getDeclaredMethod("run");
            System.out.print(name + "=");
            Class<?>[] exceptions = method.getExceptionTypes();
            for (int i = 0; i < exceptions.length; i++) {
                if (i > 0) System.out.print(",");
                System.out.print(exceptions[i].getName());
            }
            System.out.println();
        }
    }
}

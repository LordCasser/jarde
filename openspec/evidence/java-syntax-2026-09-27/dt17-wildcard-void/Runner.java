package dt17;

import java.lang.reflect.Method;
import java.util.List;

public class Runner {
    public static void main(String[] args) throws Exception {
        for (String name : new String[] {"any", "ext", "sup", "extArray", "supArray", "raw"}) {
            Method method = Wildcards.class.getMethod(name, List.class);
            System.out.println(name + "=" + method.getGenericParameterTypes()[0].getTypeName());
        }
    }
}

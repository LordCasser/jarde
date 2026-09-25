import java.lang.reflect.Method;
import java.util.*;
public class ReflectionRunner {
    public static void main(String[] args) throws Exception {
        Class<?> c = OrdinaryParameterizedSignatures.class;
        for (String name : new String[] {"strings", "numbers", "nested", "arrays", "raw"}) {
            Method m = name.equals("strings") ? c.getMethod(name, Iterable.class) : name.equals("nested") ? c.getMethod(name, Map.class) : name.equals("arrays") ? c.getMethod(name, List[].class) : c.getMethod(name, List.class);
            System.out.println(name + " parameter=" + m.getGenericParameterTypes()[0].getTypeName() + " return=" + m.getGenericReturnType().getTypeName());
        }
        Map<String, List<Integer>> m = new HashMap<>(); m.put("x", Arrays.asList(7));
        System.out.println(OrdinaryParameterizedSignatures.nested(m).get("x").get(0));
    }
}

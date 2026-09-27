package dt18;

import java.lang.reflect.Method;
import java.util.Arrays;
import java.util.List;

public class Runner {
    public static void main(String[] args) throws Exception {
        Class<GenericSlots> type = GenericSlots.class;
        Method id = type.getDeclaredMethod("id", List.class);
        Method empty = type.getDeclaredMethod("empty");
        Method raw = type.getDeclaredMethod("raw", List.class);
        System.out.println("field=" + type.getDeclaredField("names").getGenericType());
        System.out.println("id-param=" + id.getGenericParameterTypes()[0]);
        System.out.println("id-return=" + id.getGenericReturnType());
        System.out.println("empty-return=" + empty.getGenericReturnType());
        System.out.println("raw-field=" + type.getDeclaredField("raw").getGenericType());
        System.out.println("raw-param=" + raw.getGenericParameterTypes()[0]);
        System.out.println("raw-return=" + raw.getGenericReturnType());
        GenericSlots slots = new GenericSlots();
        System.out.println("values=" + slots.id(Arrays.asList("ok")).get(0) + ":" + (slots.empty() == null));
    }
}

package fieldsig;

import java.lang.reflect.Field;

public final class StandaloneFieldReflectionRunner {
    public static void main(String[] args) throws Exception {
        StandaloneFieldBoundary value = new StandaloneFieldBoundary();
        System.out.println("values=" + value.names + "," + value.current + "," + value.vector + "," + value.sink);
        System.out.println("constants=" + StandaloneFieldBoundary.TOKEN + "," + StandaloneFieldBoundary.LABEL);
        for (String name : new String[] { "names", "numbers", "current", "vector", "sink", "TOKEN", "LABEL" }) {
            Field field = StandaloneFieldBoundary.class.getDeclaredField(name);
            System.out.println(name + "=" + field.getGenericType().getTypeName());
        }
    }
}

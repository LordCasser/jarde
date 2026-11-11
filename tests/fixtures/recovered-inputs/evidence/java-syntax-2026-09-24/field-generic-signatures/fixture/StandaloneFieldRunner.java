package fieldsig;

import java.lang.reflect.Field;
import java.util.List;

public final class StandaloneFieldRunner {
    private static void printType(String name) throws Exception {
        Field field = StandaloneFieldBoundary.class.getDeclaredField(name);
        System.out.println(name + "=" + field.getGenericType().getTypeName());
    }

    public static void main(String[] args) throws Exception {
        StandaloneFieldBoundary<String> value = new StandaloneFieldBoundary<String>();
        List<String> names = value.names;
        String current = value.current;
        String[] vector = value.vector;
        List<? super String> sink = value.sink;
        System.out.println("values=" + names + "," + current + "," + vector + "," + sink);
        System.out.println("constants=" + StandaloneFieldBoundary.TOKEN + "," + StandaloneFieldBoundary.LABEL);
        for (String name : new String[] { "names", "numbers", "current", "vector", "sink", "TOKEN", "LABEL" }) {
            printType(name);
        }
    }
}

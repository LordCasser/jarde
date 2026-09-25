package fieldsig;

import java.lang.reflect.Field;
import java.util.List;

public final class FieldReflectionRunner {
    public static void main(String[] args) throws Exception {
        FieldSignatureBoundary value = new FieldSignatureBoundary("seed");
        value.names.add("name");
        value.sink.add("sink");
        value.current = "current";
        value.vector[0] = "vector";
        System.out.println("values=" + value.names.get(0) + "," + value.current + "," +
                value.vector[0] + "," + value.readNumbers().get(0) + "," + value.sink.get(0));
        for (String name : new String[] { "names", "numbers", "current", "vector", "sink", "TOKEN", "LABEL" }) {
            Field field = FieldSignatureBoundary.class.getDeclaredField(name);
            System.out.println(name + "=" + field.getGenericType().getTypeName());
        }
    }
}

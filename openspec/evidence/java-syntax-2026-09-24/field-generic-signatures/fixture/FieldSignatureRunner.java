package fieldsig;

import java.lang.reflect.Field;

public final class FieldSignatureRunner {
    private static void printType(Class<?> owner, String name) throws Exception {
        Field field = owner.getDeclaredField(name);
        System.out.println(name + "=" + field.getGenericType().getTypeName());
    }

    public static void main(String[] args) throws Exception {
        FieldSignatureBoundary<String> value = new FieldSignatureBoundary<String>("seed");
        value.names.add("name");
        value.sink.add("sink");
        value.current = "current";
        value.vector[0] = "vector";
        String firstName = value.names.get(0);
        System.out.println("values=" + firstName + "," + value.current + "," +
                value.vector[0] + "," + value.readNumbers().get(0) + "," + value.sink.get(0));
        System.out.println("constants=" + FieldSignatureBoundary.TOKEN + "," + FieldSignatureBoundary.LABEL);
        for (String name : new String[] { "names", "numbers", "current", "vector", "sink", "TOKEN", "LABEL" }) {
            printType(FieldSignatureBoundary.class, name);
        }
    }
}

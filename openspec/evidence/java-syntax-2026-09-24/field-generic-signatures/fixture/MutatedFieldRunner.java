package fieldsig;

import java.lang.reflect.Field;

public final class MutatedFieldRunner {
    public static void main(String[] args) throws Exception {
        FieldSignatureBoundary<String> value = new FieldSignatureBoundary<String>("ok");
        System.out.println("value=" + value.current);
        try {
            Field field = FieldSignatureBoundary.class.getDeclaredField(args[0]);
            System.out.println("generic=" + field.getGenericType().getTypeName());
        } catch (Throwable failure) {
            System.out.println("genericError=" + failure.getClass().getName());
        }
    }
}

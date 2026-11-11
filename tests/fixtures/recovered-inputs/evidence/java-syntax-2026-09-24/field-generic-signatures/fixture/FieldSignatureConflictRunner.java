package fieldsig;

import java.lang.reflect.Field;
import java.util.List;

public final class FieldSignatureConflictRunner {
    public static void main(String[] args) throws Exception {
        FieldSignatureConflict value = new FieldSignatureConflict();
        value.addInteger();
        Field field = FieldSignatureConflict.class.getField("items");
        Object stored = ((List<?>) field.get(value)).get(0);
        System.out.println("generic=" + field.getGenericType().getTypeName());
        System.out.println("value=" + stored + "," + stored.getClass().getName());
    }
}

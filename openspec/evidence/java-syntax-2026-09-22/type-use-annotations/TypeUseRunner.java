import java.lang.reflect.AnnotatedType;
import java.lang.reflect.Field;
import java.lang.reflect.Method;
import java.util.Arrays;

final class TypeUseRunner {
    private static String value(AnnotatedType type) {
        TypeMark mark = type.getAnnotation(TypeMark.class);
        return mark == null ? "null" : mark.value();
    }

    public static void main(String[] args) throws Exception {
        Field field = TypeUseSubject.class.getDeclaredField("field");
        Method value = TypeUseSubject.class.getDeclaredMethod("value");
        Method echo = TypeUseSubject.class.getDeclaredMethod("echo", String.class);
        System.out.println(value(field.getAnnotatedType()));
        System.out.println(value(value.getAnnotatedReturnType()));
        System.out.println(value(echo.getAnnotatedParameterTypes()[0]));
        System.out.println(Arrays.toString(new TypeUseSubject().echo("ok").toCharArray()));
    }
}

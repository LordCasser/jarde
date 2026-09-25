import java.lang.reflect.Field;
import java.lang.reflect.Method;

public class ReflectCheck {
    private static String names(java.lang.annotation.Annotation[] annotations) {
        if (annotations.length == 0) return "[]";
        StringBuilder result = new StringBuilder("[");
        for (int i = 0; i < annotations.length; i++) {
            if (i != 0) result.append(", ");
            result.append(annotations[i].annotationType().getSimpleName());
        }
        return result.append(']').toString();
    }

    public static void main(String[] args) throws Exception {
        Field field = ScalarCases.class.getDeclaredField("field");
        Method answer = ScalarCases.class.getDeclaredMethod("answer");
        Method echo = ScalarCases.class.getDeclaredMethod("echo", int.class);
        System.out.println("field.declaration=" + names(field.getDeclaredAnnotations()));
        System.out.println("field.type=" + names(field.getAnnotatedType().getAnnotations()));
        System.out.println("answer.declaration=" + names(answer.getDeclaredAnnotations()));
        System.out.println("answer.returnType=" + names(answer.getAnnotatedReturnType().getAnnotations()));
        System.out.println("echo.parameter=" + names(echo.getParameterAnnotations()[0]));
        System.out.println("echo.parameterType=" + names(echo.getAnnotatedParameterTypes()[0].getAnnotations()));
    }
}

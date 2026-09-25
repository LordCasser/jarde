import java.lang.reflect.AnnotatedElement;
import java.lang.reflect.Field;
import java.lang.reflect.Method;
import java.lang.reflect.Parameter;
import java.util.Arrays;

public class PlacementRunner {
    private static String marks(AnnotatedElement element) {
        return Arrays.toString(element.getAnnotationsByType(PlaceMark.class));
    }

    private static void field(String name) throws Exception {
        Field field = PlacementSubject.class.getDeclaredField(name);
        System.out.println("field " + name + " declaration=" + marks(field)
                + " type=" + marks(field.getAnnotatedType()));
    }

    private static void method(String name) throws Exception {
        Method method = PlacementSubject.class.getDeclaredMethod(name,
                name.equals("scalarMethod")
                        ? new Class<?>[] { int.class, String.class }
                        : new Class<?>[0]);
        System.out.println("method " + name + " declaration=" + marks(method)
                + " returnType=" + marks(method.getAnnotatedReturnType()));
        if (name.equals("scalarMethod")) {
            Parameter[] parameters = method.getParameters();
            for (int i = 0; i < parameters.length; i++) {
                System.out.println("parameter " + i + " declaration=" + marks(parameters[i])
                        + " type=" + marks(method.getAnnotatedParameterTypes()[i]));
            }
        }
    }

    public static void main(String[] args) throws Exception {
        field("scalarField");
        field("qualifiedField");
        method("scalarMethod");
        method("qualifiedMethod");
    }
}

import java.lang.reflect.Method;
import java.lang.reflect.Type;
import java.util.Arrays;
import java.util.List;

final class TypedIterableForeachProbe {
    static String typed(Iterable<String> values) {
        StringBuilder result = new StringBuilder();
        for (String value : values) {
            result.append(value.length()).append(',');
        }
        return result.toString();
    }

    static String objectThenCast(Iterable values) {
        StringBuilder result = new StringBuilder();
        for (Object item : values) {
            String value = (String) item;
            result.append(value.length()).append(',');
        }
        return result.toString();
    }

    @SuppressWarnings("unchecked")
    static String typedRawViaElementCast(Iterable values) {
        StringBuilder result = new StringBuilder();
        for (String value : (Iterable<String>) values) {
            result.append(value.length()).append(',');
        }
        return result.toString();
    }

    public static void main(String[] args) throws Exception {
        List<String> values = Arrays.asList("a", "bc", "def");
        System.out.println("typed=" + typed(values));
        System.out.println("objectThenCast=" + objectThenCast(values));
        System.out.println("typedRawViaElementCast=" + typedRawViaElementCast(values));
        for (String methodName : Arrays.asList("typed", "objectThenCast", "typedRawViaElementCast")) {
            Method method = TypedIterableForeachProbe.class.getDeclaredMethod(methodName, Iterable.class);
            Type parameter = method.getGenericParameterTypes()[0];
            System.out.println(methodName + "-reflect=" + parameter.getTypeName());
        }
        List<String> withNull = Arrays.asList("a", null, "bc");
        for (String methodName : Arrays.asList("typed", "objectThenCast", "typedRawViaElementCast")) {
            try {
                Method method = TypedIterableForeachProbe.class.getDeclaredMethod(methodName, Iterable.class);
                method.invoke(null, withNull);
            } catch (java.lang.reflect.InvocationTargetException exception) {
                System.out.println(methodName + "-null=" + exception.getCause().getClass().getSimpleName());
            }
        }
        Iterable badElement = Arrays.asList(Integer.valueOf(7));
        for (String methodName : Arrays.asList("objectThenCast", "typedRawViaElementCast")) {
            try {
                Method method = TypedIterableForeachProbe.class.getDeclaredMethod(methodName, Iterable.class);
                method.invoke(null, badElement);
            } catch (java.lang.reflect.InvocationTargetException exception) {
                System.out.println(methodName + "-badElement=" + exception.getCause().getClass().getSimpleName());
            }
        }
    }
}

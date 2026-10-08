import java.lang.reflect.Field;
import java.lang.reflect.Method;
import java.lang.reflect.Modifier;
import java.lang.reflect.Type;
import java.util.Arrays;
import java.util.Comparator;

public class ReceiverProbe {
    public static void main(String[] args) throws Exception {
        Class<?> cls = Class.forName(args[0]);
        Field field = cls.getField("value");
        System.out.println("field=" + field.getGenericType().getTypeName());
        Method[] methods = Arrays.stream(cls.getDeclaredMethods())
                .filter(m -> m.getName().equals("put") || m.getName().equals("observe"))
                .sorted(Comparator.comparing(Method::getName))
                .toArray(Method[]::new);
        for (Method method : methods) {
            System.out.println("method=" + method.getName() + ";type-parameters=" + method.getTypeParameters().length
                    + ";parameters=" + Arrays.toString(Arrays.stream(method.getGenericParameterTypes()).map(Type::getTypeName).toArray()));
        }
        Method put = Arrays.stream(cls.getDeclaredMethods()).filter(m -> m.getName().equals("put")).findFirst().get();
        Object receiver = cls.getConstructor().newInstance();
        Object value = new Object();
        Object[] callArgs = put.getParameterCount() == 2 ? new Object[] { receiver, value } : new Object[] { value };
        if (Modifier.isStatic(put.getModifiers())) {
            put.invoke(null, callArgs);
        } else {
            put.invoke(receiver, callArgs);
        }
        Object actual = field.get(receiver);
        System.out.println("behavior=" + actual.getClass().getName());
    }
}

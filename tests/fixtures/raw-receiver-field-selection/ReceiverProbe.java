import java.lang.reflect.Field;
import java.lang.reflect.GenericArrayType;
import java.lang.reflect.Method;
import java.lang.reflect.Modifier;
import java.lang.reflect.ParameterizedType;
import java.lang.reflect.Type;
import java.lang.reflect.TypeVariable;
import java.lang.reflect.WildcardType;
import java.util.Arrays;
import java.util.Comparator;

public class ReceiverProbe {
    private static String type(Type type, Class<?> owner, Method method) {
        if (type instanceof TypeVariable<?>) {
            TypeVariable<?> variable = (TypeVariable<?>) type;
            Object declaration = variable.getGenericDeclaration();
            String scope = declaration instanceof Class<?> ? "class:" + ((Class<?>) declaration).getName()
                    : declaration instanceof Method ? "method:" + ((Method) declaration).getName() : "other";
            boolean methodIdentity = declaration instanceof Method && method != null && declaration.equals(method);
            return variable.getName() + "{declaration=" + scope + ",owner=" + (declaration == owner)
                    + ",method=" + methodIdentity + "}";
        }
        if (type instanceof GenericArrayType) {
            return type(((GenericArrayType) type).getGenericComponentType(), owner, method) + "[]";
        }
        if (type instanceof ParameterizedType) {
            ParameterizedType p = (ParameterizedType) type;
            return type(p.getRawType(), owner, method) + "<" + join(p.getActualTypeArguments(), owner, method) + ">";
        }
        if (type instanceof WildcardType) {
            WildcardType w = (WildcardType) type;
            return "? extends " + join(w.getUpperBounds(), owner, method) + " super " + join(w.getLowerBounds(), owner, method);
        }
        return type.getTypeName();
    }

    private static String join(Type[] types, Class<?> owner, Method method) {
        String[] values = new String[types.length];
        for (int i = 0; i < types.length; i++) values[i] = type(types[i], owner, method);
        return String.join(",", values);
    }

    private static Object value(String className, Object marker) {
        if (className.equals("[Ljava.lang.Object;")) return new Object[] { marker };
        if (Number.class.isAssignableFrom(load(className))) return Integer.valueOf(17);
        return marker;
    }

    private static Class<?> load(String name) {
        try { return Class.forName(name); } catch (ClassNotFoundException e) { throw new RuntimeException(e); }
    }

    public static void main(String[] args) throws Exception {
        Class<?> cls = Class.forName(args[0]);
        Field field = cls.getField("value");
        System.out.println("field.type=" + field.getGenericType().getTypeName());
        System.out.println("field.declaring=" + field.getDeclaringClass().getName());
        System.out.println("field.identity=" + type(field.getGenericType(), field.getDeclaringClass(), null));
        Method[] methods = Arrays.stream(cls.getDeclaredMethods())
                .sorted(Comparator.comparing(Method::getName).thenComparing(Method::toGenericString))
                .toArray(Method[]::new);
        Method put = null;
        for (Method method : methods) {
            System.out.println("method=" + method.getName() + ";params="
                    + join(method.getGenericParameterTypes(), cls, method) + ";return="
                    + type(method.getGenericReturnType(), cls, method) + ";typeParameters="
                    + join(method.getTypeParameters(), cls, method));
            if (method.getName().equals("put")) put = method;
        }
        if (put == null) throw new IllegalStateException("missing put method in " + cls.getName());
        Object receiver = cls.getConstructor().newInstance();
        Object marker = new Object();
        Class<?>[] params = put.getParameterTypes();
        Object[] call = new Object[params.length];
        if (put.getName().equals("put") && cls.getSimpleName().equals("ThisReceiver")) {
            call[0] = marker;
        } else if (put.getName().equals("put") && cls.getSimpleName().equals("InstanceRawLocal")) {
            call[0] = marker;
        } else {
            for (int i = 0; i < params.length; i++) {
                if (i == 0 && !Modifier.isStatic(put.getModifiers()) && params[i] == Object.class) call[i] = marker;
                else if (params[i] == cls || params[i].isAssignableFrom(cls)) call[i] = receiver;
                else if (params[i] == long.class) call[i] = 7L;
                else if (params[i] == double.class) call[i] = 2.5d;
                else if (cls.getSimpleName().equals("NullRawParam")) call[i] = null;
                else if (cls.getSimpleName().equals("MultiFormalRawParam") && i == 1) call[i] = "key";
                else call[i] = value(params[i].getName(), marker);
            }
        }
        Object result = Modifier.isStatic(put.getModifiers()) ? put.invoke(null, call) : put.invoke(receiver, call);
        Object actual = field.get(receiver);
        System.out.println("behavior.value=" + (actual == null ? "null" : actual.getClass().getName()));
        System.out.println("behavior.array-length=" + (actual instanceof Object[] ? ((Object[]) actual).length : "n/a"));
        System.out.println("behavior.value-is-marker=" + (actual == marker));
        System.out.println("behavior.array-element-is-marker=" + (actual instanceof Object[] && ((Object[]) actual).length > 0 && ((Object[]) actual)[0] == marker));
        System.out.println("behavior.number-value=" + (actual instanceof Number ? ((Number) actual).intValue() : "n/a"));
        System.out.println("behavior.return-is-receiver=" + (result == receiver));
    }
}

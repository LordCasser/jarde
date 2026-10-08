import java.lang.reflect.Field;
import java.lang.reflect.Array;
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
    private static String methodKey(Method method) {
        StringBuilder key = new StringBuilder(method.getName()).append('(');
        for (Class<?> parameter : method.getParameterTypes()) key.append(parameter.getName()).append(';');
        return key.append(')').append(method.getReturnType().getName()).toString();
    }

    private static String declarationKey(Object declaration) {
        if (declaration instanceof Class<?>) return "class:" + ((Class<?>) declaration).getName();
        if (declaration instanceof Method) return "method:" + methodKey((Method) declaration);
        return "other:" + declaration;
    }

    private static String type(Type type, Class<?> owner, Method method) {
        if (type instanceof TypeVariable<?>) {
            TypeVariable<?> variable = (TypeVariable<?>) type;
            Object declaration = variable.getGenericDeclaration();
            boolean methodIdentity = declaration instanceof Method && method != null
                    && declaration.equals(method) && methodKey((Method) declaration).equals(methodKey(method));
            return variable.getName() + "{declaration=" + declarationKey(declaration)
                    + ",owner=" + (declaration == owner) + ",method=" + methodIdentity + "}";
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

    private static String bounds(TypeVariable<?> variable, Class<?> owner, Method method) {
        return join(variable.getBounds(), owner, method);
    }

    private static Object value(String className, Object marker) {
        Class<?> valueClass = load(className);
        if (valueClass.isArray()) {
            Object array = Array.newInstance(valueClass.getComponentType(), 1);
            if (!valueClass.getComponentType().isPrimitive()) {
                Array.set(array, 0, value(valueClass.getComponentType().getName(), marker));
            }
            return array;
        }
        if (Number.class.isAssignableFrom(valueClass)) return Integer.valueOf(17);
        return marker;
    }

    private static boolean containsMarker(Object value, Object marker) {
        if (value == marker) return true;
        if (value == null || !value.getClass().isArray()) return false;
        for (int i = 0; i < Array.getLength(value); i++) {
            if (containsMarker(Array.get(value, i), marker)) return true;
        }
        return false;
    }

    private static Class<?> load(String name) {
        try { return Class.forName(name); } catch (ClassNotFoundException e) { throw new RuntimeException(e); }
    }

    public static void main(String[] args) throws Exception {
        Class<?> cls = Class.forName(args[0]);
        Field field = cls.getField("value");
        System.out.println("field.type=" + field.getGenericType().getTypeName());
        System.out.println("field.physical-type=" + field.getType().getName());
        System.out.println("field.declaring=" + field.getDeclaringClass().getName());
        System.out.println("field.identity=" + type(field.getGenericType(), field.getDeclaringClass(), null));
        for (TypeVariable<?> variable : cls.getTypeParameters()) {
            System.out.println("class.type-parameter=" + variable.getName() + ";bounds="
                    + bounds(variable, cls, null) + ";declaration=" + declarationKey(variable.getGenericDeclaration())
                    + ";owner=" + (variable.getGenericDeclaration() == cls));
        }
        Method[] methods = Arrays.stream(cls.getDeclaredMethods())
                .sorted(Comparator.comparing(Method::getName).thenComparing(Method::toGenericString))
                .toArray(Method[]::new);
        Method put = null;
        for (Method method : methods) {
            System.out.println("method=" + method.getName() + ";params="
                    + join(method.getGenericParameterTypes(), cls, method) + ";return="
                    + type(method.getGenericReturnType(), cls, method) + ";typeParameters="
                    + join(method.getTypeParameters(), cls, method) + ";physical-params="
                    + joinClasses(method.getParameterTypes()) + ";physical-return="
                    + method.getReturnType().getName() + ";declaration=" + methodKey(method));
            for (TypeVariable<Method> variable : method.getTypeParameters()) {
                System.out.println("method.type-parameter=" + methodKey(method) + ";name=" + variable.getName()
                        + ";bounds=" + bounds(variable, cls, method) + ";declaration="
                        + declarationKey(variable.getGenericDeclaration()) + ";method-identity="
                        + variable.getGenericDeclaration().equals(method));
            }
            if (method.getName().equals("put")) put = method;
        }
        if (put == null) throw new IllegalStateException("missing put method in " + cls.getName());
        Object receiver = cls.getConstructor().newInstance();
        Object marker = new Object();
        Class<?>[] params = put.getParameterTypes();
        Object[] call = new Object[params.length];
        Object assignedInput = null;
        if (put.getName().equals("put") && cls.getSimpleName().equals("ThisReceiver")) {
            call[0] = marker;
        } else if (put.getName().equals("put") && cls.getSimpleName().equals("InstanceRawLocal")) {
            call[0] = marker;
        } else {
            for (int i = 0; i < params.length; i++) {
                if (i == 0 && !Modifier.isStatic(put.getModifiers()) && params[i] == Object.class) call[i] = marker;
                else if (params[i] == cls) call[i] = receiver;
                else if (params[i] == long.class) call[i] = 7L;
                else if (params[i] == double.class) call[i] = 2.5d;
                else if (cls.getSimpleName().equals("NullRawParam")) call[i] = null;
                else if (cls.getSimpleName().equals("MultiFormalRawParam") && i == 1) call[i] = "key";
                else call[i] = value(params[i].getName(), marker);
            }
        }
        if (!cls.getSimpleName().equals("NullRawParam") && call.length > 0) {
            assignedInput = call[call.length - 1];
        }
        Object result = Modifier.isStatic(put.getModifiers()) ? put.invoke(null, call) : put.invoke(receiver, call);
        Object actual = field.get(receiver);
        System.out.println("behavior.value=" + (actual == null ? "null" : actual.getClass().getName()));
        System.out.println("behavior.array-length=" + (actual != null && actual.getClass().isArray() ? Array.getLength(actual) : "n/a"));
        System.out.println("behavior.value-is-marker=" + (actual == marker));
        System.out.println("behavior.array-element-is-marker=" + containsMarker(actual, marker));
        System.out.println("behavior.value-is-input=" + (actual != null && actual == assignedInput));
        System.out.println("behavior.number-value=" + (actual instanceof Number ? ((Number) actual).intValue() : "n/a"));
        System.out.println("behavior.return-is-receiver=" + (result == receiver));
    }

    private static String joinClasses(Class<?>[] classes) {
        String[] values = new String[classes.length];
        for (int i = 0; i < classes.length; i++) values[i] = classes[i].getName();
        return String.join(",", values);
    }
}

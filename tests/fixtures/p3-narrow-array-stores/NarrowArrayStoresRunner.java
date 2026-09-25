import java.lang.reflect.Array;
import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;

public class NarrowArrayStoresRunner {
    private static final int[] VALUES = {
        -32769, -32768, -257, -256, -129, -128, -2, -1, 0, 1, 2,
        127, 128, 255, 256, 32767, 32768, 65535, 65536,
        Integer.MIN_VALUE, Integer.MAX_VALUE
    };

    private static String error(Throwable error) {
        return error.getClass().getName();
    }

    private static void set(Object array, Class<?> component, int index, int value) {
        if (component == byte.class) {
            Array.setByte(array, index, (byte) value);
        } else if (component == char.class) {
            Array.setChar(array, index, (char) value);
        } else if (component == short.class) {
            Array.setShort(array, index, (short) value);
        } else if (component == boolean.class) {
            Array.setBoolean(array, index, value != 0);
        } else {
            Array.setInt(array, index, value);
        }
    }

    private static String get(Object array, int index) {
        Object value = Array.get(array, index);
        if (value instanceof Character) {
            return Integer.toString(((Character) value).charValue());
        }
        return String.valueOf(value);
    }

    private static Object array(Class<?> component, int initial) {
        Object result = Array.newInstance(component, 1);
        set(result, component, 0, initial);
        return result;
    }

    private static Method directMethod(String name, Class<?> component) throws Exception {
        return NarrowArrayStores.class.getMethod(name, Array.newInstance(component, 0).getClass(), int.class, int.class);
    }

    private static Method producedMethod(String name, Class<?> component) throws Exception {
        return NarrowArrayStores.class.getMethod(name, Array.newInstance(component, 0).getClass(), int.class, int.class, boolean.class);
    }

    private static Throwable invoke(Method method, Object... arguments) {
        try {
            method.invoke(null, arguments);
            return null;
        } catch (InvocationTargetException error) {
            return error.getCause();
        } catch (Throwable error) {
            return error;
        }
    }

    private static void direct(String name, Class<?> component, int initial) throws Exception {
        Method method = directMethod("store" + name, component);
        for (int value : VALUES) {
            Object target = array(component, initial);
            Throwable error = invoke(method, target, 0, value);
            if (error == null) {
                System.out.println(name + ":value:" + value + ":stored:" + get(target, 0));
            } else {
                System.out.println(name + ":value:" + value + ":error:" + error(error));
            }
        }
        Throwable nullError = invoke(method, new Object[] { null, 0, 1 });
        if (nullError == null) {
            System.out.println(name + ":null:returned");
        } else {
            System.out.println(name + ":null:error:" + error(nullError));
        }
        Object target = array(component, initial);
        Throwable boundsError = invoke(method, target, 1, 1);
        if (boundsError == null) {
            System.out.println(name + ":oob:returned");
        } else {
            System.out.println(name + ":oob:error:" + error(boundsError));
        }
    }

    private static void produced(String name, Class<?> component, int initial) throws Exception {
        Method method = producedMethod("store" + name + "Produced", component);
        for (int value : VALUES) {
            NarrowArrayStoreEffects.reset();
            Object target = array(component, initial);
            Throwable error = invoke(method, target, 0, value, false);
            if (error == null) {
                System.out.println(name + ":value:" + value + ":stored:" + get(target, 0) + ":calls:" + NarrowArrayStoreEffects.calls);
            } else {
                System.out.println(name + ":value:" + value + ":error:" + error(error) + ":calls:" + NarrowArrayStoreEffects.calls);
            }
        }
        NarrowArrayStoreEffects.reset();
        Object failed = array(component, initial);
        Throwable producerError = invoke(method, failed, 0, 9, true);
        if (producerError == null) {
            System.out.println(name + ":producer-fail:returned:stored:" + get(failed, 0) + ":calls:" + NarrowArrayStoreEffects.calls);
        } else {
            System.out.println(name + ":producer-fail:error:" + error(producerError) + ":stored:" + get(failed, 0) + ":calls:" + NarrowArrayStoreEffects.calls);
        }
        NarrowArrayStoreEffects.reset();
        Throwable nullError = invoke(method, new Object[] { null, 0, 1, true });
        if (nullError == null) {
            System.out.println(name + ":null-producer-fail:returned:calls:" + NarrowArrayStoreEffects.calls);
        } else {
            System.out.println(name + ":null-producer-fail:error:" + error(nullError) + ":calls:" + NarrowArrayStoreEffects.calls);
        }
        NarrowArrayStoreEffects.reset();
        Throwable boundsFail = invoke(method, array(component, initial), 1, 1, true);
        if (boundsFail == null) {
            System.out.println(name + ":oob-producer-fail:returned:calls:" + NarrowArrayStoreEffects.calls);
        } else {
            System.out.println(name + ":oob-producer-fail:error:" + error(boundsFail) + ":calls:" + NarrowArrayStoreEffects.calls);
        }
        NarrowArrayStoreEffects.reset();
        Throwable boundsOk = invoke(method, array(component, initial), 1, 1, false);
        if (boundsOk == null) {
            System.out.println(name + ":oob-producer-ok:returned:calls:" + NarrowArrayStoreEffects.calls);
        } else {
            System.out.println(name + ":oob-producer-ok:error:" + error(boundsOk) + ":calls:" + NarrowArrayStoreEffects.calls);
        }
    }

    private static void ordinary() throws Exception {
        Object bytes = Array.newInstance(byte.class, 3);
        Object chars = Array.newInstance(char.class, 3);
        Object shorts = Array.newInstance(short.class, 3);
        NarrowArrayStores.class.getMethod("ordinaryByte", bytes.getClass()).invoke(null, bytes);
        NarrowArrayStores.class.getMethod("ordinaryChar", chars.getClass()).invoke(null, chars);
        NarrowArrayStores.class.getMethod("ordinaryShort", shorts.getClass()).invoke(null, shorts);
        System.out.println("ordinary:byte:" + get(bytes, 0) + "," + get(bytes, 1) + "," + get(bytes, 2));
        System.out.println("ordinary:char:" + (int) ((Character) Array.get(chars, 0)).charValue() + "," + (int) ((Character) Array.get(chars, 1)).charValue() + "," + (int) ((Character) Array.get(chars, 2)).charValue());
        System.out.println("ordinary:short:" + get(shorts, 0) + "," + get(shorts, 1) + "," + get(shorts, 2));
    }

    public static void main(String[] args) throws Exception {
        boolean original = args.length > 0 && "original".equals(args[0]);
        Class<?> component = original ? int.class : byte.class;
        direct("Byte", component, -77);
        component = original ? int.class : char.class;
        direct("Char", component, 77);
        component = original ? int.class : short.class;
        direct("Short", component, -77);
        component = original ? int.class : byte.class;
        produced("Byte", component, -77);
        component = original ? int.class : char.class;
        produced("Char", component, 77);
        component = original ? int.class : short.class;
        produced("Short", component, -77);
        ordinary();
    }
}

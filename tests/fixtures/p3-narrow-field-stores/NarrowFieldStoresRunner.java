import java.lang.reflect.Field;
import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;

public class NarrowFieldStoresRunner {
    private static final int[] VALUES = {
        -32769, -32768, -257, -256, -129, -128, -2, -1, 0, 1, 2,
        127, 128, 255, 256, 32767, 32768, 65535, 65536,
        Integer.MIN_VALUE, Integer.MAX_VALUE
    };

    private static final class Case {
        final String name;
        final String field;
        final String staticField;
        final String setter;
        final String producedSetter;
        final String onProducedSetter;
        final String staticSetter;
        final String staticProducedSetter;
        final String ordinaryMethod;
        final String ordinaryField;

        Case(String name, String field, String staticField, String setter, String producedSetter,
             String onProducedSetter, String staticSetter, String staticProducedSetter,
             String ordinaryMethod, String ordinaryField) {
            this.name = name;
            this.field = field;
            this.staticField = staticField;
            this.setter = setter;
            this.producedSetter = producedSetter;
            this.onProducedSetter = onProducedSetter;
            this.staticSetter = staticSetter;
            this.staticProducedSetter = staticProducedSetter;
            this.ordinaryMethod = ordinaryMethod;
            this.ordinaryField = ordinaryField;
        }
    }

    private static String error(Throwable error) {
        return error.getClass().getName();
    }

    private static Throwable call(Method method, Object receiver, Object... arguments) {
        try {
            method.invoke(receiver, arguments);
            return null;
        } catch (InvocationTargetException error) {
            return error.getCause();
        } catch (Throwable error) {
            return error;
        }
    }

    private static void setField(Field field, Object receiver, int value) throws IllegalAccessException {
        Class<?> type = field.getType();
        if (type == byte.class) {
            field.setByte(receiver, (byte) value);
        } else if (type == char.class) {
            field.setChar(receiver, (char) value);
        } else if (type == short.class) {
            field.setShort(receiver, (short) value);
        } else {
            field.setInt(receiver, value);
        }
    }

    private static String getField(Field field, Object receiver) throws IllegalAccessException {
        Object value = field.get(receiver);
        if (value instanceof Character) {
            return Integer.toString(((Character) value).charValue());
        }
        return String.valueOf(value);
    }

    private static Field field(String name) throws Exception {
        return NarrowFieldStores.class.getField(name);
    }

    private static Method method(String name, boolean produced) throws Exception {
        return produced
            ? NarrowFieldStores.class.getMethod(name, int.class, boolean.class)
            : NarrowFieldStores.class.getMethod(name, int.class);
    }

    private static Method onProducedMethod(String name) throws Exception {
        return NarrowFieldStores.class.getMethod(name, NarrowFieldStores.class, int.class, boolean.class);
    }

    private static void instanceDirect(Case c) throws Exception {
        Field target = field(c.field);
        Method setter = method(c.setter, false);
        for (int value : VALUES) {
            NarrowFieldStores object = new NarrowFieldStores();
            Throwable error = call(setter, object, value);
            if (error == null) {
                System.out.println(c.name + ":instance:value:" + value + ":stored:" + getField(target, object));
            } else {
                System.out.println(c.name + ":instance:value:" + value + ":error:" + error(error));
            }
        }
    }

    private static void instanceProduced(Case c) throws Exception {
        Field target = field(c.field);
        Method setter = method(c.producedSetter, true);
        for (int value : VALUES) {
            NarrowFieldStoreEffects.reset();
            NarrowFieldStores object = new NarrowFieldStores();
            Throwable error = call(setter, object, value, false);
            if (error == null) {
                System.out.println(c.name + ":instance-produced:value:" + value + ":stored:" + getField(target, object) + ":calls:" + NarrowFieldStoreEffects.calls);
            } else {
                System.out.println(c.name + ":instance-produced:value:" + value + ":error:" + error(error) + ":calls:" + NarrowFieldStoreEffects.calls);
            }
        }
        NarrowFieldStoreEffects.reset();
        NarrowFieldStores object = new NarrowFieldStores();
        setField(target, object, 77);
        Throwable producerError = call(setter, object, 9, true);
        System.out.println(c.name + ":instance-produced:producer-fail:" + (producerError == null ? "returned" : "error:" + error(producerError)) + ":stored:" + getField(target, object) + ":calls:" + NarrowFieldStoreEffects.calls);

        NarrowFieldStoreEffects.reset();
        Method onSetter = onProducedMethod(c.onProducedSetter);
        Throwable nullFail = call(onSetter, null, null, 9, true);
        System.out.println(c.name + ":instance-produced:null-fail:" + (nullFail == null ? "returned" : "error:" + error(nullFail)) + ":calls:" + NarrowFieldStoreEffects.calls);
        NarrowFieldStoreEffects.reset();
        Throwable nullOk = call(onSetter, null, null, 9, false);
        System.out.println(c.name + ":instance-produced:null-ok:" + (nullOk == null ? "returned" : "error:" + error(nullOk)) + ":calls:" + NarrowFieldStoreEffects.calls);
    }

    private static void staticDirect(Case c) throws Exception {
        Field target = field(c.staticField);
        Method setter = method(c.staticSetter, false);
        for (int value : VALUES) {
            setField(target, null, 77);
            Throwable error = call(setter, null, value);
            if (error == null) {
                System.out.println(c.name + ":static:value:" + value + ":stored:" + getField(target, null));
            } else {
                System.out.println(c.name + ":static:value:" + value + ":error:" + error(error));
            }
        }
    }

    private static void staticProduced(Case c) throws Exception {
        Field target = field(c.staticField);
        Method setter = method(c.staticProducedSetter, true);
        for (int value : VALUES) {
            setField(target, null, 77);
            NarrowFieldStoreEffects.reset();
            Throwable error = call(setter, null, value, false);
            if (error == null) {
                System.out.println(c.name + ":static-produced:value:" + value + ":stored:" + getField(target, null) + ":calls:" + NarrowFieldStoreEffects.calls);
            } else {
                System.out.println(c.name + ":static-produced:value:" + value + ":error:" + error(error) + ":calls:" + NarrowFieldStoreEffects.calls);
            }
        }
        setField(target, null, 77);
        NarrowFieldStoreEffects.reset();
        Throwable producerError = call(setter, null, 9, true);
        System.out.println(c.name + ":static-produced:producer-fail:" + (producerError == null ? "returned" : "error:" + error(producerError)) + ":stored:" + getField(target, null) + ":calls:" + NarrowFieldStoreEffects.calls);
    }

    private static void ordinary(Case c) throws Exception {
        NarrowFieldStores object = new NarrowFieldStores();
        Field target = field(c.ordinaryField);
        NarrowFieldStores.class.getMethod(c.ordinaryMethod).invoke(object);
        System.out.println(c.name + ":ordinary:stored:" + getField(target, object));
    }

    public static void main(String[] args) throws Exception {
        Case[] cases = {
            new Case("B", "byteField", "staticByteField", "setByte", "setByteProduced", "setByteProducedOn", "setStaticByte", "setStaticByteProduced", "ordinaryByteLocal", "ordinaryByteField"),
            new Case("C", "charField", "staticCharField", "setChar", "setCharProduced", "setCharProducedOn", "setStaticChar", "setStaticCharProduced", "ordinaryCharLocal", "ordinaryCharField"),
            new Case("S", "shortField", "staticShortField", "setShort", "setShortProduced", "setShortProducedOn", "setStaticShort", "setStaticShortProduced", "ordinaryShortLocal", "ordinaryShortField"),
        };
        for (Case c : cases) {
            instanceDirect(c);
            instanceProduced(c);
            staticDirect(c);
            staticProduced(c);
            ordinary(c);
        }
    }
}

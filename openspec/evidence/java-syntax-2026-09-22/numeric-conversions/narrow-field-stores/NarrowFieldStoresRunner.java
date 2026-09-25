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
        final Class<?> component;
        final String field;
        final String staticField;
        final String setter;
        final String producedSetter;
        final String onSetter;
        final String onProducedSetter;
        final String staticSetter;
        final String staticProducedSetter;
        final String ordinaryLocal;
        final String ordinaryZero;
        final String ordinaryOne;
        final String staticOrdinaryLocal;
        final String staticOrdinaryZero;
        final String staticOrdinaryOne;
        final String ordinaryField;
        final String ordinaryStaticField;

        Case(String name, Class<?> component, String field, String staticField,
             String setter, String producedSetter, String onSetter, String onProducedSetter,
             String staticSetter, String staticProducedSetter, String ordinaryLocal, String ordinaryZero, String ordinaryOne,
             String staticOrdinaryLocal, String staticOrdinaryZero, String staticOrdinaryOne,
             String ordinaryField, String ordinaryStaticField) {
            this.name = name;
            this.component = component;
            this.field = field;
            this.staticField = staticField;
            this.setter = setter;
            this.producedSetter = producedSetter;
            this.onSetter = onSetter;
            this.onProducedSetter = onProducedSetter;
            this.staticSetter = staticSetter;
            this.staticProducedSetter = staticProducedSetter;
            this.ordinaryLocal = ordinaryLocal;
            this.ordinaryZero = ordinaryZero;
            this.ordinaryOne = ordinaryOne;
            this.staticOrdinaryLocal = staticOrdinaryLocal;
            this.staticOrdinaryZero = staticOrdinaryZero;
            this.staticOrdinaryOne = staticOrdinaryOne;
            this.ordinaryField = ordinaryField;
            this.ordinaryStaticField = ordinaryStaticField;
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

    private static void setField(Field field, Object receiver, Class<?> component, int value) throws IllegalAccessException {
        if (component == byte.class) {
            field.setByte(receiver, (byte) value);
        } else if (component == char.class) {
            field.setChar(receiver, (char) value);
        } else if (component == short.class) {
            field.setShort(receiver, (short) value);
        } else if (component == boolean.class) {
            field.setBoolean(receiver, value != 0);
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

    private static Method onMethod(String name, boolean produced) throws Exception {
        return produced
            ? NarrowFieldStores.class.getMethod(name, NarrowFieldStores.class, int.class, boolean.class)
            : NarrowFieldStores.class.getMethod(name, NarrowFieldStores.class, int.class);
    }

    private static void instanceDirect(Case c, boolean original) throws Exception {
        Field field = field(c.field);
        Method setter = method(c.setter, false);
        for (int value : VALUES) {
            NarrowFieldStores object = new NarrowFieldStores();
            Throwable error = call(setter, object, value);
            if (error == null) {
                System.out.println(c.name + ":instance:value:" + value + ":stored:" + getField(field, object));
            } else {
                System.out.println(c.name + ":instance:value:" + value + ":error:" + error(error));
            }
        }
        Method nullSetter = onMethod(c.onSetter, false);
        Throwable nullError = call(nullSetter, null, new Object[] { null, 1 });
        if (nullError == null) {
            System.out.println(c.name + ":instance:null:returned");
        } else {
            System.out.println(c.name + ":instance:null:error:" + error(nullError));
        }
    }

    private static void instanceProduced(Case c, boolean original) throws Exception {
        Class<?> component = original ? int.class : c.component;
        Field field = field(c.field);
        Method setter = method(c.producedSetter, true);
        for (int value : VALUES) {
            NarrowFieldStoreEffects.reset();
            NarrowFieldStores object = new NarrowFieldStores();
            Throwable error = call(setter, object, value, false);
            if (error == null) {
                System.out.println(c.name + ":instance-produced:value:" + value + ":stored:" + getField(field, object) + ":calls:" + NarrowFieldStoreEffects.calls);
            } else {
                System.out.println(c.name + ":instance-produced:value:" + value + ":error:" + error(error) + ":calls:" + NarrowFieldStoreEffects.calls);
            }
        }
        NarrowFieldStoreEffects.reset();
        NarrowFieldStores failed = new NarrowFieldStores();
        Field failedField = field(c.field);
        setField(failedField, failed, component, 77);
        Throwable producerError = call(setter, failed, 9, true);
        if (producerError == null) {
            System.out.println(c.name + ":instance-produced:producer-fail:returned:stored:" + getField(failedField, failed) + ":calls:" + NarrowFieldStoreEffects.calls);
        } else {
            System.out.println(c.name + ":instance-produced:producer-fail:error:" + error(producerError) + ":stored:" + getField(failedField, failed) + ":calls:" + NarrowFieldStoreEffects.calls);
        }
        NarrowFieldStoreEffects.reset();
        Method nullSetter = onMethod(c.onProducedSetter, true);
        Throwable nullFail = call(nullSetter, null, new Object[] { null, 9, true });
        if (nullFail == null) {
            System.out.println(c.name + ":instance-produced:null-fail:returned:calls:" + NarrowFieldStoreEffects.calls);
        } else {
            System.out.println(c.name + ":instance-produced:null-fail:error:" + error(nullFail) + ":calls:" + NarrowFieldStoreEffects.calls);
        }
        NarrowFieldStoreEffects.reset();
        Throwable nullOk = call(nullSetter, null, new Object[] { null, 9, false });
        if (nullOk == null) {
            System.out.println(c.name + ":instance-produced:null-ok:returned:calls:" + NarrowFieldStoreEffects.calls);
        } else {
            System.out.println(c.name + ":instance-produced:null-ok:error:" + error(nullOk) + ":calls:" + NarrowFieldStoreEffects.calls);
        }
    }

    private static void staticDirect(Case c, boolean original) throws Exception {
        Class<?> component = original ? int.class : c.component;
        Field field = field(c.staticField);
        Method setter = method(c.staticSetter, false);
        for (int value : VALUES) {
            setField(field, null, component, 77);
            Throwable error = call(setter, null, value);
            if (error == null) {
                System.out.println(c.name + ":static:value:" + value + ":stored:" + getField(field, null));
            } else {
                System.out.println(c.name + ":static:value:" + value + ":error:" + error(error));
            }
        }
    }

    private static void staticProduced(Case c, boolean original) throws Exception {
        Class<?> component = original ? int.class : c.component;
        Field field = field(c.staticField);
        Method setter = method(c.staticProducedSetter, true);
        for (int value : VALUES) {
            setField(field, null, component, 77);
            NarrowFieldStoreEffects.reset();
            Throwable error = call(setter, null, value, false);
            if (error == null) {
                System.out.println(c.name + ":static-produced:value:" + value + ":stored:" + getField(field, null) + ":calls:" + NarrowFieldStoreEffects.calls);
            } else {
                System.out.println(c.name + ":static-produced:value:" + value + ":error:" + error(error) + ":calls:" + NarrowFieldStoreEffects.calls);
            }
        }
        setField(field, null, component, 77);
        NarrowFieldStoreEffects.reset();
        Throwable producerError = call(setter, null, 9, true);
        if (producerError == null) {
            System.out.println(c.name + ":static-produced:producer-fail:returned:stored:" + getField(field, null) + ":calls:" + NarrowFieldStoreEffects.calls);
        } else {
            System.out.println(c.name + ":static-produced:producer-fail:error:" + error(producerError) + ":stored:" + getField(field, null) + ":calls:" + NarrowFieldStoreEffects.calls);
        }
    }

    private static void ordinary(Case c) throws Exception {
        NarrowFieldStores object = new NarrowFieldStores();
        Field instanceField = field(c.ordinaryField);
        Field staticField = field(c.ordinaryStaticField);
        NarrowFieldStores.class.getMethod(c.ordinaryLocal).invoke(object);
        System.out.println(c.name + ":ordinary-instance:local:" + getField(instanceField, object));
        NarrowFieldStores.class.getMethod(c.ordinaryZero).invoke(object);
        System.out.println(c.name + ":ordinary-instance:zero:" + getField(instanceField, object));
        NarrowFieldStores.class.getMethod(c.ordinaryOne).invoke(object);
        System.out.println(c.name + ":ordinary-instance:one:" + getField(instanceField, object));
        NarrowFieldStores.class.getMethod(c.staticOrdinaryLocal).invoke(null);
        System.out.println(c.name + ":ordinary-static:local:" + getField(staticField, null));
        NarrowFieldStores.class.getMethod(c.staticOrdinaryZero).invoke(null);
        System.out.println(c.name + ":ordinary-static:zero:" + getField(staticField, null));
        NarrowFieldStores.class.getMethod(c.staticOrdinaryOne).invoke(null);
        System.out.println(c.name + ":ordinary-static:one:" + getField(staticField, null));
    }

    public static void main(String[] args) throws Exception {
        boolean original = args.length > 0 && "original".equals(args[0]);
        Case[] cases = {
            new Case("B", byte.class, "byteField", "staticByteField", "setByte", "setByteProduced", "setByteOn", "setByteProducedOn", "setStaticByte", "setStaticByteProduced", "ordinaryByteLocal", "ordinaryByteZero", "ordinaryByteOne", "ordinaryStaticByteLocal", "ordinaryStaticByteZero", "ordinaryStaticByteOne", "ordinaryByteField", "ordinaryStaticByteField"),
            new Case("C", char.class, "charField", "staticCharField", "setChar", "setCharProduced", "setCharOn", "setCharProducedOn", "setStaticChar", "setStaticCharProduced", "ordinaryCharLocal", "ordinaryCharZero", "ordinaryCharOne", "ordinaryStaticCharLocal", "ordinaryStaticCharZero", "ordinaryStaticCharOne", "ordinaryCharField", "ordinaryStaticCharField"),
            new Case("S", short.class, "shortField", "staticShortField", "setShort", "setShortProduced", "setShortOn", "setShortProducedOn", "setStaticShort", "setStaticShortProduced", "ordinaryShortLocal", "ordinaryShortZero", "ordinaryShortOne", "ordinaryStaticShortLocal", "ordinaryStaticShortZero", "ordinaryStaticShortOne", "ordinaryShortField", "ordinaryStaticShortField"),
            new Case("Z", boolean.class, "booleanField", "staticBooleanField", "setBoolean", "setBooleanProduced", "setBooleanOn", "setBooleanProducedOn", "setStaticBoolean", "setStaticBooleanProduced", "ordinaryBooleanLocal", "ordinaryBooleanFalse", "ordinaryBooleanTrue", "ordinaryStaticBooleanLocal", "ordinaryStaticBooleanFalse", "ordinaryStaticBooleanTrue", "ordinaryBooleanField", "ordinaryStaticBooleanField")
        };
        for (Case c : cases) {
            instanceDirect(c, original);
            instanceProduced(c, original);
            staticDirect(c, original);
            staticProduced(c, original);
            ordinary(c);
        }
    }
}

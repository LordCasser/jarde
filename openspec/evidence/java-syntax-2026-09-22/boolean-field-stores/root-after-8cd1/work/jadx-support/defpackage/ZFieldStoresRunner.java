package defpackage;

import java.lang.reflect.Field;
import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;

public class ZFieldStoresRunner {
    private static final int[] VALUES = {
        -2, -1, 0, 1, 2, Integer.MIN_VALUE, Integer.MAX_VALUE
    };

    private static Throwable call(String name, Class<?>[] parameterTypes, Object... arguments) {
        try {
            Method method = ZFieldStores.class.getMethod(name, parameterTypes);
            method.invoke(null, arguments);
            return null;
        } catch (InvocationTargetException error) {
            return error.getCause();
        } catch (Throwable error) {
            return error;
        }
    }

    private static Throwable callInstance(ZFieldStores receiver, String name, Class<?>[] parameterTypes, Object... arguments) {
        try {
            Method method = ZFieldStores.class.getMethod(name, parameterTypes);
            method.invoke(receiver, arguments);
            return null;
        } catch (InvocationTargetException error) {
            return error.getCause();
        } catch (Throwable error) {
            return error;
        }
    }

    private static String read(String name, Object receiver) throws Exception {
        Field field = ZFieldStores.class.getField(name);
        Object value = field.get(receiver);
        if (value instanceof Boolean) {
            return Boolean.toString(((Boolean) value).booleanValue());
        }
        return String.valueOf(value);
    }

    private static void prepare(String name, Object receiver) throws Exception {
        Field field = ZFieldStores.class.getField(name);
        if (field.getType() == boolean.class) {
            field.setBoolean(receiver, true);
        } else {
            field.setInt(receiver, 77);
        }
    }

    private static String type(Throwable error) {
        return error == null ? "none" : error.getClass().getName();
    }

    private static void directAndProduced() throws Exception {
        for (int value : VALUES) {
            ZFieldStores instance = new ZFieldStores();
            Throwable error = callInstance(instance, "putInstance", new Class<?>[] { int.class }, Integer.valueOf(value));
            System.out.println("instance:" + value + ":" + type(error) + ":" + read("instanceFlag", instance));

            ZFieldStores.putStatic(77);
            error = call("putStatic", new Class<?>[] { int.class }, Integer.valueOf(value));
            System.out.println("static:" + value + ":" + type(error) + ":" + read("staticFlag", null));

            instance = new ZFieldStores();
            ZFieldStoreEffects.reset();
            error = callInstance(instance, "putInstanceProduced", new Class<?>[] { int.class, boolean.class }, Integer.valueOf(value), Boolean.FALSE);
            System.out.println("instance-produced:" + value + ":" + type(error) + ":" + read("instanceFlag", instance) + ":calls:" + ZFieldStoreEffects.calls);

            ZFieldStores.putStatic(77);
            ZFieldStoreEffects.reset();
            error = call("putStaticProduced", new Class<?>[] { int.class, boolean.class }, Integer.valueOf(value), Boolean.FALSE);
            System.out.println("static-produced:" + value + ":" + type(error) + ":" + read("staticFlag", null) + ":calls:" + ZFieldStoreEffects.calls);
        }

        ZFieldStores instance = new ZFieldStores();
        prepare("instanceFlag", instance);
        ZFieldStoreEffects.reset();
        Throwable failed = callInstance(instance, "putInstanceProduced", new Class<?>[] { int.class, boolean.class }, Integer.valueOf(9), Boolean.TRUE);
        System.out.println("instance-produced:fail:" + type(failed) + ":" + read("instanceFlag", instance) + ":calls:" + ZFieldStoreEffects.calls);

        prepare("staticFlag", null);
        ZFieldStoreEffects.reset();
        failed = call("putStaticProduced", new Class<?>[] { int.class, boolean.class }, Integer.valueOf(9), Boolean.TRUE);
        System.out.println("static-produced:fail:" + type(failed) + ":" + read("staticFlag", null) + ":calls:" + ZFieldStoreEffects.calls);
    }

    private static void receivers() throws Exception {
        Throwable error = call("putOn", new Class<?>[] { ZFieldStores.class, int.class }, null, Integer.valueOf(1));
        System.out.println("null-direct:" + type(error));

        ZFieldStoreEffects.reset();
        error = call("putProducedOn", new Class<?>[] { ZFieldStores.class, int.class, boolean.class }, null, Integer.valueOf(9), Boolean.TRUE);
        System.out.println("null-produced-fail:" + type(error) + ":calls:" + ZFieldStoreEffects.calls);

        ZFieldStoreEffects.reset();
        error = call("putProducedOn", new Class<?>[] { ZFieldStores.class, int.class, boolean.class }, null, Integer.valueOf(9), Boolean.FALSE);
        System.out.println("null-produced-normal:" + type(error) + ":calls:" + ZFieldStoreEffects.calls);

        ZFieldStores receiver = new ZFieldStores();
        ZFieldStoreEffects.reset();
        error = call("putProducedOn", new Class<?>[] { ZFieldStores.class, int.class, boolean.class }, receiver, Integer.valueOf(-1), Boolean.FALSE);
        System.out.println("receiver-produced:-1:" + type(error) + ":" + read("instanceFlag", receiver) + ":calls:" + ZFieldStoreEffects.calls);
    }

    private static void ordinaryBoolean() throws Exception {
        ZFieldStores receiver = new ZFieldStores();
        receiver.putOrdinaryInstance(false);
        System.out.println("ordinary-instance:false:" + read("ordinaryInstanceFlag", receiver));
        receiver.putOrdinaryInstance(true);
        System.out.println("ordinary-instance:true:" + read("ordinaryInstanceFlag", receiver));
        receiver.putOrdinaryInstanceTrue();
        System.out.println("ordinary-instance:literal:" + read("ordinaryInstanceFlag", receiver));

        ZFieldStores.putOrdinaryStatic(false);
        System.out.println("ordinary-static:false:" + read("ordinaryStaticFlag", null));
        ZFieldStores.putOrdinaryStatic(true);
        System.out.println("ordinary-static:true:" + read("ordinaryStaticFlag", null));
        ZFieldStores.putOrdinaryStaticTrue();
        System.out.println("ordinary-static:literal:" + read("ordinaryStaticFlag", null));
    }

    public static void main(String[] args) throws Exception {
        directAndProduced();
        receivers();
        ordinaryBoolean();
    }
}

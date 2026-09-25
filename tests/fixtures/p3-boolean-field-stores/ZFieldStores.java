public class ZFieldStores {
    public int instanceFlag;
    public static int staticFlag;

    public boolean ordinaryInstanceFlag;
    public static boolean ordinaryStaticFlag;

    public void putInstance(int value) {
        this.instanceFlag = value;
    }

    public void putInstanceProduced(int value, boolean fail) {
        this.instanceFlag = ZFieldStoreEffects.value(value, fail);
    }

    public static void putStatic(int value) {
        staticFlag = value;
    }

    public static void putStaticProduced(int value, boolean fail) {
        staticFlag = ZFieldStoreEffects.value(value, fail);
    }

    public static void putOn(ZFieldStores receiver, int value) {
        receiver.instanceFlag = value;
    }

    public static void putProducedOn(ZFieldStores receiver, int value, boolean fail) {
        receiver.instanceFlag = ZFieldStoreEffects.value(value, fail);
    }

    public void putOrdinaryInstance(boolean value) {
        ordinaryInstanceFlag = value;
    }

    public void putOrdinaryInstanceTrue() {
        ordinaryInstanceFlag = true;
    }

    public static void putOrdinaryStatic(boolean value) {
        ordinaryStaticFlag = value;
    }

    public static void putOrdinaryStaticTrue() {
        ordinaryStaticFlag = true;
    }
}

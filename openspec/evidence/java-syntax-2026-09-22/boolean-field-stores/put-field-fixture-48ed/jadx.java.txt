package defpackage;

/* JADX INFO: loaded from: ZFieldStores.class */
public class ZFieldStores {
    public boolean instanceFlag;
    public static boolean staticFlag;
    public boolean ordinaryInstanceFlag;
    public static boolean ordinaryStaticFlag;

    /* JADX WARN: Multi-variable type inference failed */
    public void putInstance(int i) {
        this.instanceFlag = i;
    }

    /* JADX WARN: Type inference failed for: r1v1, types: [boolean, int] */
    public void putInstanceProduced(int i, boolean z) {
        this.instanceFlag = ZFieldStoreEffects.value(i, z);
    }

    /* JADX WARN: Multi-variable type inference failed */
    public static void putStatic(int i) {
        staticFlag = i;
    }

    /* JADX WARN: Type inference failed for: r0v1, types: [boolean, int] */
    public static void putStaticProduced(int i, boolean z) {
        staticFlag = ZFieldStoreEffects.value(i, z);
    }

    /* JADX WARN: Multi-variable type inference failed */
    public static void putOn(ZFieldStores zFieldStores, int i) {
        zFieldStores.instanceFlag = i;
    }

    /* JADX WARN: Type inference failed for: r1v1, types: [boolean, int] */
    public static void putProducedOn(ZFieldStores zFieldStores, int i, boolean z) {
        zFieldStores.instanceFlag = ZFieldStoreEffects.value(i, z);
    }

    public void putOrdinaryInstance(boolean z) {
        this.ordinaryInstanceFlag = z;
    }

    public void putOrdinaryInstanceTrue() {
        this.ordinaryInstanceFlag = true;
    }

    public static void putOrdinaryStatic(boolean z) {
        ordinaryStaticFlag = z;
    }

    public static void putOrdinaryStaticTrue() {
        ordinaryStaticFlag = true;
    }
}

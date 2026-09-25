public class NarrowFieldStores {
    public int byteField;
    public int charField;
    public int shortField;
    public int booleanField;

    public static int staticByteField;
    public static int staticCharField;
    public static int staticShortField;
    public static int staticBooleanField;

    public byte ordinaryByteField;
    public char ordinaryCharField;
    public short ordinaryShortField;
    public boolean ordinaryBooleanField;

    public static byte ordinaryStaticByteField;
    public static char ordinaryStaticCharField;
    public static short ordinaryStaticShortField;
    public static boolean ordinaryStaticBooleanField;

    public void setByte(int value) {
        this.byteField = value;
    }

    public void setChar(int value) {
        this.charField = value;
    }

    public void setShort(int value) {
        this.shortField = value;
    }

    public void setBoolean(int value) {
        this.booleanField = value;
    }

    public void setByteProduced(int value, boolean fail) {
        this.byteField = NarrowFieldStoreEffects.value(value, fail);
    }

    public static void setByteOn(NarrowFieldStores object, int value) {
        object.byteField = value;
    }

    public static void setByteProducedOn(NarrowFieldStores object, int value, boolean fail) {
        object.byteField = NarrowFieldStoreEffects.value(value, fail);
    }

    public void setCharProduced(int value, boolean fail) {
        this.charField = NarrowFieldStoreEffects.value(value, fail);
    }

    public static void setCharOn(NarrowFieldStores object, int value) {
        object.charField = value;
    }

    public static void setCharProducedOn(NarrowFieldStores object, int value, boolean fail) {
        object.charField = NarrowFieldStoreEffects.value(value, fail);
    }

    public void setShortProduced(int value, boolean fail) {
        this.shortField = NarrowFieldStoreEffects.value(value, fail);
    }

    public static void setShortOn(NarrowFieldStores object, int value) {
        object.shortField = value;
    }

    public static void setShortProducedOn(NarrowFieldStores object, int value, boolean fail) {
        object.shortField = NarrowFieldStoreEffects.value(value, fail);
    }

    public void setBooleanProduced(int value, boolean fail) {
        this.booleanField = NarrowFieldStoreEffects.value(value, fail);
    }

    public static void setBooleanOn(NarrowFieldStores object, int value) {
        object.booleanField = value;
    }

    public static void setBooleanProducedOn(NarrowFieldStores object, int value, boolean fail) {
        object.booleanField = NarrowFieldStoreEffects.value(value, fail);
    }

    public static void setStaticByte(int value) {
        staticByteField = value;
    }

    public static void setStaticChar(int value) {
        staticCharField = value;
    }

    public static void setStaticShort(int value) {
        staticShortField = value;
    }

    public static void setStaticBoolean(int value) {
        staticBooleanField = value;
    }

    public static void setStaticByteProduced(int value, boolean fail) {
        staticByteField = NarrowFieldStoreEffects.value(value, fail);
    }

    public static void setStaticCharProduced(int value, boolean fail) {
        staticCharField = NarrowFieldStoreEffects.value(value, fail);
    }

    public static void setStaticShortProduced(int value, boolean fail) {
        staticShortField = NarrowFieldStoreEffects.value(value, fail);
    }

    public static void setStaticBooleanProduced(int value, boolean fail) {
        staticBooleanField = NarrowFieldStoreEffects.value(value, fail);
    }

    public void ordinaryByteLocal() {
        byte value = 1;
        ordinaryByteField = value;
    }

    public void ordinaryByteZero() {
        ordinaryByteField = 0;
    }

    public void ordinaryByteOne() {
        ordinaryByteField = 1;
    }

    public void ordinaryCharLocal() {
        char value = 1;
        ordinaryCharField = value;
    }

    public void ordinaryCharZero() {
        ordinaryCharField = 0;
    }

    public void ordinaryCharOne() {
        ordinaryCharField = 1;
    }

    public void ordinaryShortLocal() {
        short value = 1;
        ordinaryShortField = value;
    }

    public void ordinaryShortZero() {
        ordinaryShortField = 0;
    }

    public void ordinaryShortOne() {
        ordinaryShortField = 1;
    }

    public void ordinaryBooleanLocal() {
        boolean value = true;
        ordinaryBooleanField = value;
    }

    public void ordinaryBooleanFalse() {
        ordinaryBooleanField = false;
    }

    public void ordinaryBooleanTrue() {
        ordinaryBooleanField = true;
    }

    public static void ordinaryStaticByteLocal() {
        byte value = 1;
        ordinaryStaticByteField = value;
    }

    public static void ordinaryStaticByteZero() {
        ordinaryStaticByteField = 0;
    }

    public static void ordinaryStaticByteOne() {
        ordinaryStaticByteField = 1;
    }

    public static void ordinaryStaticCharLocal() {
        char value = 1;
        ordinaryStaticCharField = value;
    }

    public static void ordinaryStaticCharZero() {
        ordinaryStaticCharField = 0;
    }

    public static void ordinaryStaticCharOne() {
        ordinaryStaticCharField = 1;
    }

    public static void ordinaryStaticShortLocal() {
        short value = 1;
        ordinaryStaticShortField = value;
    }

    public static void ordinaryStaticShortZero() {
        ordinaryStaticShortField = 0;
    }

    public static void ordinaryStaticShortOne() {
        ordinaryStaticShortField = 1;
    }

    public static void ordinaryStaticBooleanLocal() {
        boolean value = true;
        ordinaryStaticBooleanField = value;
    }

    public static void ordinaryStaticBooleanFalse() {
        ordinaryStaticBooleanField = false;
    }

    public static void ordinaryStaticBooleanTrue() {
        ordinaryStaticBooleanField = true;
    }
}

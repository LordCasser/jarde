public class NarrowFieldStores {
    public int byteField;
    public int charField;
    public int shortField;

    public static int staticByteField;
    public static int staticCharField;
    public static int staticShortField;

    public byte ordinaryByteField;
    public char ordinaryCharField;
    public short ordinaryShortField;

    public void setByte(int value) {
        this.byteField = value;
    }

    public void setChar(int value) {
        this.charField = value;
    }

    public void setShort(int value) {
        this.shortField = value;
    }

    public void setByteProduced(int value, boolean fail) {
        this.byteField = NarrowFieldStoreEffects.value(value, fail);
    }

    public void setCharProduced(int value, boolean fail) {
        this.charField = NarrowFieldStoreEffects.value(value, fail);
    }

    public void setShortProduced(int value, boolean fail) {
        this.shortField = NarrowFieldStoreEffects.value(value, fail);
    }

    public static void setByteProducedOn(NarrowFieldStores object, int value, boolean fail) {
        object.byteField = NarrowFieldStoreEffects.value(value, fail);
    }

    public static void setCharProducedOn(NarrowFieldStores object, int value, boolean fail) {
        object.charField = NarrowFieldStoreEffects.value(value, fail);
    }

    public static void setShortProducedOn(NarrowFieldStores object, int value, boolean fail) {
        object.shortField = NarrowFieldStoreEffects.value(value, fail);
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

    public static void setStaticByteProduced(int value, boolean fail) {
        staticByteField = NarrowFieldStoreEffects.value(value, fail);
    }

    public static void setStaticCharProduced(int value, boolean fail) {
        staticCharField = NarrowFieldStoreEffects.value(value, fail);
    }

    public static void setStaticShortProduced(int value, boolean fail) {
        staticShortField = NarrowFieldStoreEffects.value(value, fail);
    }

    public void ordinaryByteLocal() {
        byte value = 1;
        ordinaryByteField = value;
    }

    public void ordinaryCharLocal() {
        char value = 1;
        ordinaryCharField = value;
    }

    public void ordinaryShortLocal() {
        short value = 1;
        ordinaryShortField = value;
    }
}

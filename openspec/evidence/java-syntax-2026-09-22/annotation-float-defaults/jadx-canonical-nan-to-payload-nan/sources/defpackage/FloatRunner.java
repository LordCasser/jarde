package defpackage;

/* JADX INFO: loaded from: FloatRunner.class */
class FloatRunner {
    FloatRunner() {
    }

    public static void main(String[] strArr) throws Exception {
        System.out.println(Integer.toHexString(Float.floatToRawIntBits(((Float) FloatDefaults.class.getDeclaredMethod("negativeZero", new Class[0]).getDefaultValue()).floatValue())));
        System.out.println(Long.toHexString(Double.doubleToRawLongBits(((Double) FloatDefaults.class.getDeclaredMethod("subnormal", new Class[0]).getDefaultValue()).doubleValue())));
        System.out.println(Integer.toHexString(Float.floatToRawIntBits(((Float) FloatDefaults.class.getDeclaredMethod("positiveInfinity", new Class[0]).getDefaultValue()).floatValue())));
        System.out.println(Long.toHexString(Double.doubleToRawLongBits(((Double) FloatDefaults.class.getDeclaredMethod("canonicalNaN", new Class[0]).getDefaultValue()).doubleValue())));
    }
}

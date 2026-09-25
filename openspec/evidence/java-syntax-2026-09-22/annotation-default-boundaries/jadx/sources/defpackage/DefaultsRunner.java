package defpackage;

import java.util.Arrays;

/* JADX INFO: loaded from: DefaultsRunner.class */
class DefaultsRunner {
    DefaultsRunner() {
    }

    static void check(boolean z, String str) {
        if (!z) {
            throw new AssertionError(str);
        }
    }

    public static void main(String[] strArr) throws Exception {
        check(((Inner) Defaults.class.getMethod("nested", new Class[0]).getDefaultValue()).count() == 9, "nested annotation");
        Inner[] innerArr = (Inner[]) Defaults.class.getMethod("nestedArray", new Class[0]).getDefaultValue();
        check(innerArr.length == 2 && innerArr[0].count() == 2 && innerArr[1].count() == 4, "nested array");
        check(Defaults.class.getMethod("klass", new Class[0]).getDefaultValue().equals(String[].class), "class literal");
        check(Defaults.class.getMethod("tone", new Class[0]).getDefaultValue() == Tone.LOUD, "enum");
        check(((int[]) Defaults.class.getMethod("empty", new Class[0]).getDefaultValue()).length == 0, "empty array");
        check(Arrays.equals((int[]) Defaults.class.getMethod("numbers", new Class[0]).getDefaultValue(), new int[]{3, 5}), "numbers");
        check(Float.floatToRawIntBits(((Float) Defaults.class.getMethod("negativeZeroFloat", new Class[0]).getDefaultValue()).floatValue()) == Integer.MIN_VALUE, "float negative zero");
        check(Double.doubleToRawLongBits(((Double) Defaults.class.getMethod("negativeZeroDouble", new Class[0]).getDefaultValue()).doubleValue()) == Long.MIN_VALUE, "double negative zero");
        check(((Float) Defaults.class.getMethod("positiveInfinity", new Class[0]).getDefaultValue()).isInfinite(), "float infinity");
        check(((Double) Defaults.class.getMethod("notANumber", new Class[0]).getDefaultValue()).isNaN(), "double NaN");
        System.out.println("reflection defaults: PASS");
    }
}

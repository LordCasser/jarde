public class ConstructorPrimitiveConversionControls {
    static int markInt(String tag, int value) {
        System.out.print(tag);
        System.out.print(":");
        System.out.println(value);
        return value;
    }

    static long markLong(String tag, long value) {
        System.out.print(tag);
        System.out.print(":");
        System.out.println(value);
        return value;
    }

    static float markFloat(String tag, float value) {
        System.out.print(tag);
        System.out.print(":");
        System.out.println(value);
        return value;
    }

    static double markDouble(String tag, double value) {
        System.out.print(tag);
        System.out.print(":");
        System.out.println(value);
        return value;
    }

    static Byte wrapperByte(int value) {
        return new Byte((byte) markInt("wrapper-byte", value));
    }

    static Short wrapperShort(int value) {
        return new Short((short) markInt("wrapper-short", value));
    }

    static Long wrapperLong(int value) {
        return new Long((long) markInt("wrapper-long", value));
    }

    static Float wrapperFloat(long value) {
        return new Float((float) markLong("wrapper-float", value));
    }

    static Double wrapperDouble(float value) {
        return new Double((double) markFloat("wrapper-double", value));
    }

    static Integer integerContrast(int value) {
        return new Integer(markInt("integer-contrast", value));
    }

    static Number[] boxedArray(int value) {
        return new Number[]{
            new Byte((byte) markInt("array-byte", value)),
            new Short((short) markInt("array-short", value)),
            new Integer(markInt("array-integer", value)),
            new Long((long) markInt("array-long", value)),
            new Float((float) markInt("array-float", value)),
            new Double((double) markInt("array-double", value))
        };
    }

    static Long ordinaryReturnNew(int value) {
        return new Long((long) markInt("return-new", value));
    }

    static PrimitiveLongPair storedLocalReuse(int value) {
        int stored = markInt("stored-local", value);
        return new PrimitiveLongPair((long) stored, (long) stored);
    }

    static Long intToLong(int value) {
        return new Long((long) markInt("i2l", value));
    }

    static Float intToFloat(int value) {
        return new Float((float) markInt("i2f", value));
    }

    static Double intToDouble(int value) {
        return new Double((double) markInt("i2d", value));
    }

    static Byte intToByte(int value) {
        return new Byte((byte) markInt("i2b", value));
    }

    static Character intToCharacter(int value) {
        return new Character((char) markInt("i2c", value));
    }

    static Short intToShort(int value) {
        return new Short((short) markInt("i2s", value));
    }

    static Integer longToInt(long value) {
        return new Integer((int) markLong("l2i", value));
    }

    static Float longToFloat(long value) {
        return new Float((float) markLong("l2f", value));
    }

    static Double longToDouble(long value) {
        return new Double((double) markLong("l2d", value));
    }

    static Integer floatToInt(float value) {
        return new Integer((int) markFloat("f2i", value));
    }

    static Long floatToLong(float value) {
        return new Long((long) markFloat("f2l", value));
    }

    static Double floatToDouble(float value) {
        return new Double((double) markFloat("f2d", value));
    }

    static Integer doubleToInt(double value) {
        return new Integer((int) markDouble("d2i", value));
    }

    static Long doubleToLong(double value) {
        return new Long((long) markDouble("d2l", value));
    }

    static Float doubleToFloat(double value) {
        return new Float((float) markDouble("d2f", value));
    }

    static Long longViaFloat(long value) {
        return new Long((long) (float) markLong("l2f-f2l", value));
    }

    static Long longViaDouble(long value) {
        return new Long((long) (double) markLong("l2d-d2l", value));
    }

    static Double doubleViaFloat(double value) {
        return new Double((double) (float) markDouble("d2f-f2d", value));
    }

    static void show(Object value, boolean expectedType) {
        System.out.println(expectedType);
        System.out.println(value);
    }

    public static void main(String[] args) {
        Byte wrapperByte = wrapperByte(130);
        show(wrapperByte, ((Object) wrapperByte) instanceof Byte);
        Short wrapperShort = wrapperShort(65537);
        show(wrapperShort, ((Object) wrapperShort) instanceof Short);
        Long wrapperLong = wrapperLong(7);
        show(wrapperLong, ((Object) wrapperLong) instanceof Long);
        Float wrapperFloat = wrapperFloat(16777217L);
        show(wrapperFloat, ((Object) wrapperFloat) instanceof Float);
        Double wrapperDouble = wrapperDouble(16777216.0f);
        show(wrapperDouble, ((Object) wrapperDouble) instanceof Double);
        Integer integer = integerContrast(7);
        show(integer, ((Object) integer) instanceof Integer);

        Number[] boxed = boxedArray(130);
        Object boxedByte = boxed[0];
        show(boxedByte, boxedByte instanceof Byte);
        Object boxedShort = boxed[1];
        show(boxedShort, boxedShort instanceof Short);
        Object boxedInteger = boxed[2];
        show(boxedInteger, boxedInteger instanceof Integer);
        Object boxedLong = boxed[3];
        show(boxedLong, boxedLong instanceof Long);
        Object boxedFloat = boxed[4];
        show(boxedFloat, boxedFloat instanceof Float);
        Object boxedDouble = boxed[5];
        show(boxedDouble, boxedDouble instanceof Double);

        Long returnNew = ordinaryReturnNew(9);
        show(returnNew, ((Object) returnNew) instanceof Long);
        PrimitiveLongPair pair = storedLocalReuse(11);
        System.out.println(pair.left);
        System.out.println(pair.right);

        Byte i2b = intToByte(130);
        show(i2b, ((Object) i2b) instanceof Byte);
        Character i2c = intToCharacter(65);
        show(i2c, ((Object) i2c) instanceof Character);
        Short i2s = intToShort(65537);
        show(i2s, ((Object) i2s) instanceof Short);
        Long i2l = intToLong(16777217);
        show(i2l, ((Object) i2l) instanceof Long);
        Float i2f = intToFloat(16777217);
        show(i2f, ((Object) i2f) instanceof Float);
        Double i2d = intToDouble(16777217);
        show(i2d, ((Object) i2d) instanceof Double);

        Integer l2i = longToInt(4294967297L);
        show(l2i, ((Object) l2i) instanceof Integer);
        Float l2f = longToFloat(16777217L);
        show(l2f, ((Object) l2f) instanceof Float);
        Double l2d = longToDouble(9007199254740993L);
        show(l2d, ((Object) l2d) instanceof Double);

        Integer f2i = floatToInt(3.75f);
        show(f2i, ((Object) f2i) instanceof Integer);
        Long f2l = floatToLong(3.75f);
        show(f2l, ((Object) f2l) instanceof Long);
        Double f2d = floatToDouble(16777216.0f);
        show(f2d, ((Object) f2d) instanceof Double);

        Integer d2i = doubleToInt(3.75d);
        show(d2i, ((Object) d2i) instanceof Integer);
        Long d2l = doubleToLong(3.75d);
        show(d2l, ((Object) d2l) instanceof Long);
        Float d2f = doubleToFloat(16777217.0d);
        show(d2f, ((Object) d2f) instanceof Float);

        Long lRoundTrip = longViaFloat(16777217L);
        show(lRoundTrip, ((Object) lRoundTrip) instanceof Long);
        Long lDoubleRoundTrip = longViaDouble(9007199254740993L);
        show(lDoubleRoundTrip, ((Object) lDoubleRoundTrip) instanceof Long);
        Double dRoundTrip = doubleViaFloat(16777217.0d);
        show(dRoundTrip, ((Object) dRoundTrip) instanceof Double);
    }
}

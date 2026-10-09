package defpackage;

/* JADX INFO: loaded from: javac8-original-two-class.jar:ConstructorPrimitiveConversionControls.class */
public class ConstructorPrimitiveConversionControls {
    static int markInt(String str, int i) {
        System.out.print(str);
        System.out.print(":");
        System.out.println(i);
        return i;
    }

    static long markLong(String str, long j) {
        System.out.print(str);
        System.out.print(":");
        System.out.println(j);
        return j;
    }

    static float markFloat(String str, float f) {
        System.out.print(str);
        System.out.print(":");
        System.out.println(f);
        return f;
    }

    static double markDouble(String str, double d) {
        System.out.print(str);
        System.out.print(":");
        System.out.println(d);
        return d;
    }

    static Byte wrapperByte(int i) {
        return new Byte((byte) markInt("wrapper-byte", i));
    }

    static Short wrapperShort(int i) {
        return new Short((short) markInt("wrapper-short", i));
    }

    static Long wrapperLong(int i) {
        return new Long(markInt("wrapper-long", i));
    }

    static Float wrapperFloat(long j) {
        return new Float(markLong("wrapper-float", j));
    }

    static Double wrapperDouble(float f) {
        return new Double(markFloat("wrapper-double", f));
    }

    static Integer integerContrast(int i) {
        return new Integer(markInt("integer-contrast", i));
    }

    static Number[] boxedArray(int i) {
        return new Number[]{new Byte((byte) markInt("array-byte", i)), new Short((short) markInt("array-short", i)), new Integer(markInt("array-integer", i)), new Long(markInt("array-long", i)), new Float(markInt("array-float", i)), new Double(markInt("array-double", i))};
    }

    static Long ordinaryReturnNew(int i) {
        return new Long(markInt("return-new", i));
    }

    static PrimitiveLongPair storedLocalReuse(int i) {
        int iMarkInt = markInt("stored-local", i);
        return new PrimitiveLongPair(iMarkInt, iMarkInt);
    }

    static Long intToLong(int i) {
        return new Long(markInt("i2l", i));
    }

    static Float intToFloat(int i) {
        return new Float(markInt("i2f", i));
    }

    static Double intToDouble(int i) {
        return new Double(markInt("i2d", i));
    }

    static Byte intToByte(int i) {
        return new Byte((byte) markInt("i2b", i));
    }

    static Character intToCharacter(int i) {
        return new Character((char) markInt("i2c", i));
    }

    static Short intToShort(int i) {
        return new Short((short) markInt("i2s", i));
    }

    static Integer longToInt(long j) {
        return new Integer((int) markLong("l2i", j));
    }

    static Float longToFloat(long j) {
        return new Float(markLong("l2f", j));
    }

    static Double longToDouble(long j) {
        return new Double(markLong("l2d", j));
    }

    static Integer floatToInt(float f) {
        return new Integer((int) markFloat("f2i", f));
    }

    static Long floatToLong(float f) {
        return new Long((long) markFloat("f2l", f));
    }

    static Double floatToDouble(float f) {
        return new Double(markFloat("f2d", f));
    }

    static Integer doubleToInt(double d) {
        return new Integer((int) markDouble("d2i", d));
    }

    static Long doubleToLong(double d) {
        return new Long((long) markDouble("d2l", d));
    }

    static Float doubleToFloat(double d) {
        return new Float((float) markDouble("d2f", d));
    }

    static Long longViaFloat(long j) {
        return new Long(markLong("l2f-f2l", j));
    }

    static Long longViaDouble(long j) {
        return new Long(markLong("l2d-d2l", j));
    }

    static Double doubleViaFloat(double d) {
        return new Double((float) markDouble("d2f-f2d", d));
    }

    static void show(Object obj, boolean z) {
        System.out.println(z);
        System.out.println(obj);
    }

    public static void main(String[] strArr) {
        Byte bWrapperByte = wrapperByte(130);
        show(bWrapperByte, bWrapperByte instanceof Byte);
        Short shWrapperShort = wrapperShort(65537);
        show(shWrapperShort, shWrapperShort instanceof Short);
        Long lWrapperLong = wrapperLong(7);
        show(lWrapperLong, lWrapperLong instanceof Long);
        Float fWrapperFloat = wrapperFloat(16777217L);
        show(fWrapperFloat, fWrapperFloat instanceof Float);
        Double dWrapperDouble = wrapperDouble(1.6777216E7f);
        show(dWrapperDouble, dWrapperDouble instanceof Double);
        Integer numIntegerContrast = integerContrast(7);
        show(numIntegerContrast, numIntegerContrast instanceof Integer);
        Number[] numberArrBoxedArray = boxedArray(130);
        Number number = numberArrBoxedArray[0];
        show(number, number instanceof Byte);
        Number number2 = numberArrBoxedArray[1];
        show(number2, number2 instanceof Short);
        Number number3 = numberArrBoxedArray[2];
        show(number3, number3 instanceof Integer);
        Number number4 = numberArrBoxedArray[3];
        show(number4, number4 instanceof Long);
        Number number5 = numberArrBoxedArray[4];
        show(number5, number5 instanceof Float);
        Number number6 = numberArrBoxedArray[5];
        show(number6, number6 instanceof Double);
        Long lOrdinaryReturnNew = ordinaryReturnNew(9);
        show(lOrdinaryReturnNew, lOrdinaryReturnNew instanceof Long);
        PrimitiveLongPair primitiveLongPairStoredLocalReuse = storedLocalReuse(11);
        System.out.println(primitiveLongPairStoredLocalReuse.left);
        System.out.println(primitiveLongPairStoredLocalReuse.right);
        Byte bIntToByte = intToByte(130);
        show(bIntToByte, bIntToByte instanceof Byte);
        Character chIntToCharacter = intToCharacter(65);
        show(chIntToCharacter, chIntToCharacter instanceof Character);
        Short shIntToShort = intToShort(65537);
        show(shIntToShort, shIntToShort instanceof Short);
        Long lIntToLong = intToLong(16777217);
        show(lIntToLong, lIntToLong instanceof Long);
        Float fIntToFloat = intToFloat(16777217);
        show(fIntToFloat, fIntToFloat instanceof Float);
        Double dIntToDouble = intToDouble(16777217);
        show(dIntToDouble, dIntToDouble instanceof Double);
        Integer numLongToInt = longToInt(4294967297L);
        show(numLongToInt, numLongToInt instanceof Integer);
        Float fLongToFloat = longToFloat(16777217L);
        show(fLongToFloat, fLongToFloat instanceof Float);
        Double dLongToDouble = longToDouble(9007199254740993L);
        show(dLongToDouble, dLongToDouble instanceof Double);
        Integer numFloatToInt = floatToInt(3.75f);
        show(numFloatToInt, numFloatToInt instanceof Integer);
        Long lFloatToLong = floatToLong(3.75f);
        show(lFloatToLong, lFloatToLong instanceof Long);
        Double dFloatToDouble = floatToDouble(1.6777216E7f);
        show(dFloatToDouble, dFloatToDouble instanceof Double);
        Integer numDoubleToInt = doubleToInt(3.75d);
        show(numDoubleToInt, numDoubleToInt instanceof Integer);
        Long lDoubleToLong = doubleToLong(3.75d);
        show(lDoubleToLong, lDoubleToLong instanceof Long);
        Float fDoubleToFloat = doubleToFloat(1.6777217E7d);
        show(fDoubleToFloat, fDoubleToFloat instanceof Float);
        Long lLongViaFloat = longViaFloat(16777217L);
        show(lLongViaFloat, lLongViaFloat instanceof Long);
        Long lLongViaDouble = longViaDouble(9007199254740993L);
        show(lLongViaDouble, lLongViaDouble instanceof Long);
        Double dDoubleViaFloat = doubleViaFloat(1.6777217E7d);
        show(dDoubleViaFloat, dDoubleViaFloat instanceof Double);
    }
}

// jarde: presentation of `ConstructorPrimitiveConversionControls` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class ConstructorPrimitiveConversionControls extends java.lang.Object {
    public ConstructorPrimitiveConversionControls() {
        // @method <init>()V
        // @declaration a constructor of `ConstructorPrimitiveConversionControls`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static int markInt(java.lang.String arg0, int arg1) {
        // @method markInt(Ljava/lang/String;I)I
        // @declaration a static method of `ConstructorPrimitiveConversionControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.print(arg0);
        java.lang.System.out.print(":");
        java.lang.System.out.println(arg1);
        return arg1;
    }

    static long markLong(java.lang.String arg0, long arg1) {
        // @method markLong(Ljava/lang/String;J)J
        // @declaration a static method of `ConstructorPrimitiveConversionControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.print(arg0);
        java.lang.System.out.print(":");
        java.lang.System.out.println(arg1);
        return arg1;
    }

    static float markFloat(java.lang.String arg0, float arg1) {
        // @method markFloat(Ljava/lang/String;F)F
        // @declaration a static method of `ConstructorPrimitiveConversionControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.print(arg0);
        java.lang.System.out.print(":");
        java.lang.System.out.println(arg1);
        return arg1;
    }

    static double markDouble(java.lang.String arg0, double arg1) {
        // @method markDouble(Ljava/lang/String;D)D
        // @declaration a static method of `ConstructorPrimitiveConversionControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.print(arg0);
        java.lang.System.out.print(":");
        java.lang.System.out.println(arg1);
        return arg1;
    }

    static java.lang.Byte wrapperByte(int arg0) {
        // @method wrapperByte(I)Ljava/lang/Byte;
        // @declaration a static method of `ConstructorPrimitiveConversionControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return new java.lang.Byte((byte) markInt("wrapper-byte", arg0));
    }

    static java.lang.Short wrapperShort(int arg0) {
        // @method wrapperShort(I)Ljava/lang/Short;
        // @declaration a static method of `ConstructorPrimitiveConversionControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return new java.lang.Short((short) markInt("wrapper-short", arg0));
    }

    static java.lang.Long wrapperLong(int arg0) {
        // @method wrapperLong(I)Ljava/lang/Long;
        // @declaration a static method of `ConstructorPrimitiveConversionControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return new java.lang.Long((long) markInt("wrapper-long", arg0));
    }

    static java.lang.Float wrapperFloat(long arg0) {
        // @method wrapperFloat(J)Ljava/lang/Float;
        // @declaration a static method of `ConstructorPrimitiveConversionControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return new java.lang.Float((float) markLong("wrapper-float", arg0));
    }

    static java.lang.Double wrapperDouble(float arg0) {
        // @method wrapperDouble(F)Ljava/lang/Double;
        // @declaration a static method of `ConstructorPrimitiveConversionControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return new java.lang.Double((double) markFloat("wrapper-double", arg0));
    }

    static java.lang.Integer integerContrast(int arg0) {
        // @method integerContrast(I)Ljava/lang/Integer;
        // @declaration a static method of `ConstructorPrimitiveConversionControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return new java.lang.Integer(markInt("integer-contrast", arg0));
    }

    static java.lang.Number[] boxedArray(int arg0) {
        // @method boxedArray(I)[Ljava/lang/Number;
        // @declaration a static method of `ConstructorPrimitiveConversionControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return new java.lang.Number[]{new java.lang.Byte((byte) markInt("array-byte", arg0)), new java.lang.Short((short) markInt("array-short", arg0)), new java.lang.Integer(markInt("array-integer", arg0)), new java.lang.Long((long) markInt("array-long", arg0)), new java.lang.Float((float) markInt("array-float", arg0)), new java.lang.Double((double) markInt("array-double", arg0))};
    }

    static java.lang.Long ordinaryReturnNew(int arg0) {
        // @method ordinaryReturnNew(I)Ljava/lang/Long;
        // @declaration a static method of `ConstructorPrimitiveConversionControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return new java.lang.Long((long) markInt("return-new", arg0));
    }

    static PrimitiveLongPair storedLocalReuse(int arg0) {
        // @method storedLocalReuse(I)LPrimitiveLongPair;
        // @declaration a static method of `ConstructorPrimitiveConversionControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        int local1 = markInt("stored-local", arg0);
        return new PrimitiveLongPair((long) local1, (long) local1);
    }

    static java.lang.Long intToLong(int arg0) {
        // @method intToLong(I)Ljava/lang/Long;
        // @declaration a static method of `ConstructorPrimitiveConversionControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return new java.lang.Long((long) markInt("i2l", arg0));
    }

    static java.lang.Float intToFloat(int arg0) {
        // @method intToFloat(I)Ljava/lang/Float;
        // @declaration a static method of `ConstructorPrimitiveConversionControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return new java.lang.Float((float) markInt("i2f", arg0));
    }

    static java.lang.Double intToDouble(int arg0) {
        // @method intToDouble(I)Ljava/lang/Double;
        // @declaration a static method of `ConstructorPrimitiveConversionControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return new java.lang.Double((double) markInt("i2d", arg0));
    }

    static java.lang.Byte intToByte(int arg0) {
        // @method intToByte(I)Ljava/lang/Byte;
        // @declaration a static method of `ConstructorPrimitiveConversionControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return new java.lang.Byte((byte) markInt("i2b", arg0));
    }

    static java.lang.Character intToCharacter(int arg0) {
        // @method intToCharacter(I)Ljava/lang/Character;
        // @declaration a static method of `ConstructorPrimitiveConversionControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return new java.lang.Character((char) markInt("i2c", arg0));
    }

    static java.lang.Short intToShort(int arg0) {
        // @method intToShort(I)Ljava/lang/Short;
        // @declaration a static method of `ConstructorPrimitiveConversionControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return new java.lang.Short((short) markInt("i2s", arg0));
    }

    static java.lang.Integer longToInt(long arg0) {
        // @method longToInt(J)Ljava/lang/Integer;
        // @declaration a static method of `ConstructorPrimitiveConversionControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return new java.lang.Integer((int) markLong("l2i", arg0));
    }

    static java.lang.Float longToFloat(long arg0) {
        // @method longToFloat(J)Ljava/lang/Float;
        // @declaration a static method of `ConstructorPrimitiveConversionControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return new java.lang.Float((float) markLong("l2f", arg0));
    }

    static java.lang.Double longToDouble(long arg0) {
        // @method longToDouble(J)Ljava/lang/Double;
        // @declaration a static method of `ConstructorPrimitiveConversionControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return new java.lang.Double((double) markLong("l2d", arg0));
    }

    static java.lang.Integer floatToInt(float arg0) {
        // @method floatToInt(F)Ljava/lang/Integer;
        // @declaration a static method of `ConstructorPrimitiveConversionControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return new java.lang.Integer((int) markFloat("f2i", arg0));
    }

    static java.lang.Long floatToLong(float arg0) {
        // @method floatToLong(F)Ljava/lang/Long;
        // @declaration a static method of `ConstructorPrimitiveConversionControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return new java.lang.Long((long) markFloat("f2l", arg0));
    }

    static java.lang.Double floatToDouble(float arg0) {
        // @method floatToDouble(F)Ljava/lang/Double;
        // @declaration a static method of `ConstructorPrimitiveConversionControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return new java.lang.Double((double) markFloat("f2d", arg0));
    }

    static java.lang.Integer doubleToInt(double arg0) {
        // @method doubleToInt(D)Ljava/lang/Integer;
        // @declaration a static method of `ConstructorPrimitiveConversionControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return new java.lang.Integer((int) markDouble("d2i", arg0));
    }

    static java.lang.Long doubleToLong(double arg0) {
        // @method doubleToLong(D)Ljava/lang/Long;
        // @declaration a static method of `ConstructorPrimitiveConversionControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return new java.lang.Long((long) markDouble("d2l", arg0));
    }

    static java.lang.Float doubleToFloat(double arg0) {
        // @method doubleToFloat(D)Ljava/lang/Float;
        // @declaration a static method of `ConstructorPrimitiveConversionControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return new java.lang.Float((float) markDouble("d2f", arg0));
    }

    static java.lang.Long longViaFloat(long arg0) {
        // @method longViaFloat(J)Ljava/lang/Long;
        // @declaration a static method of `ConstructorPrimitiveConversionControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return new java.lang.Long((long) (float) markLong("l2f-f2l", arg0));
    }

    static java.lang.Long longViaDouble(long arg0) {
        // @method longViaDouble(J)Ljava/lang/Long;
        // @declaration a static method of `ConstructorPrimitiveConversionControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return new java.lang.Long((long) (double) markLong("l2d-d2l", arg0));
    }

    static java.lang.Double doubleViaFloat(double arg0) {
        // @method doubleViaFloat(D)Ljava/lang/Double;
        // @declaration a static method of `ConstructorPrimitiveConversionControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return new java.lang.Double((double) (float) markDouble("d2f-f2d", arg0));
    }

    static void show(java.lang.Object arg0, boolean arg1) {
        // @method show(Ljava/lang/Object;Z)V
        // @declaration a static method of `ConstructorPrimitiveConversionControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println(arg1);
        java.lang.System.out.println(arg0);
        return;
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `ConstructorPrimitiveConversionControls`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.Byte local1 = wrapperByte(130);
        show((java.lang.Object) local1, (java.lang.Object) local1 instanceof java.lang.Byte);
        java.lang.Short local2 = wrapperShort(65537);
        show((java.lang.Object) local2, (java.lang.Object) local2 instanceof java.lang.Short);
        java.lang.Long local3 = wrapperLong(7);
        show((java.lang.Object) local3, (java.lang.Object) local3 instanceof java.lang.Long);
        java.lang.Float local4 = wrapperFloat(16777217L);
        show((java.lang.Object) local4, (java.lang.Object) local4 instanceof java.lang.Float);
        java.lang.Double local5 = wrapperDouble(0x1.000000p24f);
        show((java.lang.Object) local5, (java.lang.Object) local5 instanceof java.lang.Double);
        java.lang.Integer local6 = integerContrast(7);
        show((java.lang.Object) local6, (java.lang.Object) local6 instanceof java.lang.Integer);
        java.lang.Number[] local7 = boxedArray(130);
        java.lang.Number local8 = local7[0];
        show((java.lang.Object) local8, (java.lang.Object) local8 instanceof java.lang.Byte);
        java.lang.Number local9 = local7[1];
        show((java.lang.Object) local9, (java.lang.Object) local9 instanceof java.lang.Short);
        java.lang.Number local10 = local7[2];
        show((java.lang.Object) local10, (java.lang.Object) local10 instanceof java.lang.Integer);
        java.lang.Number local11 = local7[3];
        show((java.lang.Object) local11, (java.lang.Object) local11 instanceof java.lang.Long);
        java.lang.Number local12 = local7[4];
        show((java.lang.Object) local12, (java.lang.Object) local12 instanceof java.lang.Float);
        java.lang.Number local13 = local7[5];
        show((java.lang.Object) local13, (java.lang.Object) local13 instanceof java.lang.Double);
        java.lang.Long local14 = ordinaryReturnNew(9);
        show((java.lang.Object) local14, (java.lang.Object) local14 instanceof java.lang.Long);
        PrimitiveLongPair local15 = storedLocalReuse(11);
        java.lang.System.out.println(local15.left);
        java.lang.System.out.println(local15.right);
        java.lang.Byte local16 = intToByte(130);
        show((java.lang.Object) local16, (java.lang.Object) local16 instanceof java.lang.Byte);
        java.lang.Character local17 = intToCharacter(65);
        show((java.lang.Object) local17, (java.lang.Object) local17 instanceof java.lang.Character);
        java.lang.Short local18 = intToShort(65537);
        show((java.lang.Object) local18, (java.lang.Object) local18 instanceof java.lang.Short);
        java.lang.Long local19 = intToLong(16777217);
        show((java.lang.Object) local19, (java.lang.Object) local19 instanceof java.lang.Long);
        java.lang.Float local20 = intToFloat(16777217);
        show((java.lang.Object) local20, (java.lang.Object) local20 instanceof java.lang.Float);
        java.lang.Double local21 = intToDouble(16777217);
        show((java.lang.Object) local21, (java.lang.Object) local21 instanceof java.lang.Double);
        java.lang.Integer local22 = longToInt(4294967297L);
        show((java.lang.Object) local22, (java.lang.Object) local22 instanceof java.lang.Integer);
        java.lang.Float local23 = longToFloat(16777217L);
        show((java.lang.Object) local23, (java.lang.Object) local23 instanceof java.lang.Float);
        java.lang.Double local24 = longToDouble(9007199254740993L);
        show((java.lang.Object) local24, (java.lang.Object) local24 instanceof java.lang.Double);
        java.lang.Integer local25 = floatToInt(0x1.e00000p1f);
        show((java.lang.Object) local25, (java.lang.Object) local25 instanceof java.lang.Integer);
        java.lang.Long local26 = floatToLong(0x1.e00000p1f);
        show((java.lang.Object) local26, (java.lang.Object) local26 instanceof java.lang.Long);
        java.lang.Double local27 = floatToDouble(0x1.000000p24f);
        show((java.lang.Object) local27, (java.lang.Object) local27 instanceof java.lang.Double);
        java.lang.Integer local28 = doubleToInt(0x1.e000000000000p1d);
        show((java.lang.Object) local28, (java.lang.Object) local28 instanceof java.lang.Integer);
        java.lang.Long local29 = doubleToLong(0x1.e000000000000p1d);
        show((java.lang.Object) local29, (java.lang.Object) local29 instanceof java.lang.Long);
        java.lang.Float local30 = doubleToFloat(0x1.0000010000000p24d);
        show((java.lang.Object) local30, (java.lang.Object) local30 instanceof java.lang.Float);
        java.lang.Long local31 = longViaFloat(16777217L);
        show((java.lang.Object) local31, (java.lang.Object) local31 instanceof java.lang.Long);
        java.lang.Long local32 = longViaDouble(9007199254740993L);
        show((java.lang.Object) local32, (java.lang.Object) local32 instanceof java.lang.Long);
        java.lang.Double local33 = doubleViaFloat(0x1.0000010000000p24d);
        show((java.lang.Object) local33, (java.lang.Object) local33 instanceof java.lang.Double);
        return;
    }
}

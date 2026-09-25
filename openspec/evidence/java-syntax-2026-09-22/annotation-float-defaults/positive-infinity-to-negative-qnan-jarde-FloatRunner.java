// jarde: presentation of `FloatRunner` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
class FloatRunner extends java.lang.Object {
    FloatRunner() {
        // @method <init>()V
        // @declaration a constructor of `FloatRunner`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static void main(java.lang.String[] arg0) throws java.lang.Exception {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `FloatRunner`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) java.lang.Integer.toHexString(java.lang.Float.floatToRawIntBits(((java.lang.Float) FloatDefaults.class.getDeclaredMethod("negativeZero", new java.lang.Class[0]).getDefaultValue()).floatValue())));
        java.lang.System.out.println((java.lang.String) java.lang.Long.toHexString(java.lang.Double.doubleToRawLongBits(((java.lang.Double) FloatDefaults.class.getDeclaredMethod("subnormal", new java.lang.Class[0]).getDefaultValue()).doubleValue())));
        java.lang.System.out.println((java.lang.String) java.lang.Integer.toHexString(java.lang.Float.floatToRawIntBits(((java.lang.Float) FloatDefaults.class.getDeclaredMethod("positiveInfinity", new java.lang.Class[0]).getDefaultValue()).floatValue())));
        java.lang.System.out.println((java.lang.String) java.lang.Long.toHexString(java.lang.Double.doubleToRawLongBits(((java.lang.Double) FloatDefaults.class.getDeclaredMethod("canonicalNaN", new java.lang.Class[0]).getDefaultValue()).doubleValue())));
        return;
    }
}

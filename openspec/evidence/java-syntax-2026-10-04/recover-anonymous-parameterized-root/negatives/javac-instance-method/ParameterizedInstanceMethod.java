public final class ParameterizedInstanceMethod extends java.lang.Object {
    static java.lang.String observed;

    public ParameterizedInstanceMethod() {
        // @method <init>()V
        // @declaration a constructor of `ParameterizedInstanceMethod`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    private Base create(java.lang.String arg1) {
        // @method create(Ljava/lang/String;)LBase;
        // @declaration an instance method of `ParameterizedInstanceMethod`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        return new ParameterizedInstanceMethod$1(this, arg1);
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `ParameterizedInstanceMethod`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        new ParameterizedInstanceMethod().create("captured-value");
        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append("observed=").append(ParameterizedInstanceMethod.observed).toString());
        return;
    }
}

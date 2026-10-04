public final class ParameterizedMultiParams extends java.lang.Object {
    static java.lang.String observed;

    public ParameterizedMultiParams() {
        // @method <init>()V
        // @declaration a constructor of `ParameterizedMultiParams`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    private static Base create(java.lang.String arg0, int arg1) {
        // @method create(Ljava/lang/String;I)LBase;
        // @declaration a static method of `ParameterizedMultiParams`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        return new ParameterizedMultiParams$1(arg0);
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `ParameterizedMultiParams`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        create("captured-value", 7);
        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append("observed=").append(ParameterizedMultiParams.observed).toString());
        return;
    }
}

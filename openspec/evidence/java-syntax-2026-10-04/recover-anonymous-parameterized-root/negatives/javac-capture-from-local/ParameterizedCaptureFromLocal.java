public final class ParameterizedCaptureFromLocal extends java.lang.Object {
    static java.lang.String observed;

    public ParameterizedCaptureFromLocal() {
        // @method <init>()V
        // @declaration a constructor of `ParameterizedCaptureFromLocal`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    private static Base create(java.lang.String arg0) {
        // @method create(Ljava/lang/String;)LBase;
        // @declaration a static method of `ParameterizedCaptureFromLocal`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.String local1 = arg0;
        return new ParameterizedCaptureFromLocal$1(local1);
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `ParameterizedCaptureFromLocal`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        create("captured-value");
        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append("observed=").append(ParameterizedCaptureFromLocal.observed).toString());
        return;
    }
}

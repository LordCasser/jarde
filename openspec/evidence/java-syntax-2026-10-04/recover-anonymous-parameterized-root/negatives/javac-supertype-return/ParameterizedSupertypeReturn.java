public final class ParameterizedSupertypeReturn extends java.lang.Object {
    public ParameterizedSupertypeReturn() {
        // @method <init>()V
        // @declaration a constructor of `ParameterizedSupertypeReturn`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    private static Renderer create(java.lang.String arg0) {
        // @method create(Ljava/lang/String;)LRenderer;
        // @declaration a static method of `ParameterizedSupertypeReturn`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        return new ParameterizedSupertypeReturn$1(arg0);
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `ParameterizedSupertypeReturn`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) create("captured-value").render());
        return;
    }
}

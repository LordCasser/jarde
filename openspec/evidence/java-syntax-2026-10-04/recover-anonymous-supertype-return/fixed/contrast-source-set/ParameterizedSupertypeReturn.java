// jarde: presentation of `ParameterizedSupertypeReturn` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class ParameterizedSupertypeReturn extends java.lang.Object {
    public ParameterizedSupertypeReturn() {
        // @method <init>()V
        // @declaration a constructor of `ParameterizedSupertypeReturn`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    private static Renderer create(java.lang.String arg0) {
        return new Base() {
            public java.lang.String render() {
                return new java.lang.StringBuilder().append("r:").append(arg0).toString();
            }
        };
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `ParameterizedSupertypeReturn`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) create("captured-value").render());
        return;
    }
}

// jarde: presentation of `AnonymousSuperDirect` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class AnonymousSuperDirect extends java.lang.Object {
    private static int effects;

    public AnonymousSuperDirect() {
        // @method <init>()V
        // @declaration a constructor of `AnonymousSuperDirect`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    private static int next() {
        // @method next()I
        // @declaration a static method of `AnonymousSuperDirect`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        AnonymousSuperDirect.effects = AnonymousSuperDirect.effects + 1;
        return AnonymousSuperDirect.effects == 1 ? 7 : 2;
    }

    private static Base make() {
        // @method make()LBase;
        // @declaration a static method of `AnonymousSuperDirect`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        return new AnonymousSuperDirect$1(next(), next());
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `AnonymousSuperDirect`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append(make().sum()).append(":").append(AnonymousSuperDirect.effects).toString());
        return;
    }
}

// jarde: presentation of `Inner` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class Inner extends java.lang.Object {
    static java.lang.Object observed;

    int f;

    public Inner() {
        // @method <init>()V
        // @declaration a constructor of `Inner`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.f = 37;
        return;
    }

    java.lang.Runnable make() {
        return new java.lang.Runnable() {
            public void run() {
                Inner.observed = Inner.this;
                Inner.this.f += 1;
                return;
            }
        };
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `Inner`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        Inner local1 = new Inner();
        local1.make().run();
        java.lang.System.out.println(Inner.observed.equals((java.lang.Object) local1));
        java.lang.System.out.println(local1.f);
        return;
    }
}

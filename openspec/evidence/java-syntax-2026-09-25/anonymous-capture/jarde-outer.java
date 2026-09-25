// jarde: presentation of `AnonymousProbe` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class AnonymousProbe extends java.lang.Object {
    private int state;

    public AnonymousProbe() {
        // @method <init>()V
        // @declaration a constructor of `AnonymousProbe`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.state = 2;
        return;
    }

    public AnonymousProbe$Action make(int arg1) {
        // @method make(I)LAnonymousProbe$Action;
        // @declaration an instance method of `AnonymousProbe`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        int local2 = arg1 + 1;
        return new AnonymousProbe$1(this, local2);
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `AnonymousProbe`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println(new AnonymousProbe().make(3).run(4));
        return;
    }

    static int access$000(AnonymousProbe arg0) {
        // @method access$000(LAnonymousProbe;)I
        // @declaration a static method of `AnonymousProbe`, member flags 0x1008
        // recovered from bytecode; presentation is not claimed to compile
        return arg0.state;
    }
}

// jarde: presentation of `AnonymousSuperMixedDirect` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class AnonymousSuperMixedDirect extends java.lang.Object {
    private static final java.lang.StringBuilder EVENTS = new java.lang.StringBuilder();

    public AnonymousSuperMixedDirect() {
        // @method <init>()V
        // @declaration a constructor of `AnonymousSuperMixedDirect`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static void event(java.lang.String arg0) {
        // @method event(Ljava/lang/String;)V
        // @declaration a static method of `AnonymousSuperMixedDirect`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        if (AnonymousSuperMixedDirect.EVENTS.length() != 0) {
            AnonymousSuperMixedDirect.EVENTS.append('|');
        }
        AnonymousSuperMixedDirect.EVENTS.append(arg0);
        return;
    }

    private static java.lang.String text(java.lang.String arg0, java.lang.String arg1) {
        // @method text(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `AnonymousSuperMixedDirect`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        event("arg:" + arg0);
        return arg1;
    }

    private static int number(java.lang.String arg0, int arg1) {
        // @method number(Ljava/lang/String;I)I
        // @declaration a static method of `AnonymousSuperMixedDirect`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        event("arg:" + arg0);
        return arg1;
    }

    private static java.lang.String capture() {
        // @method capture()Ljava/lang/String;
        // @declaration a static method of `AnonymousSuperMixedDirect`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        event("capture");
        return "captured";
    }

    static Base make() {
        java.lang.String local0 = capture();
        return new Base((java.lang.String) text("super-label", "explicit"), number("super-value", 17)) {
            java.lang.String render() {
                AnonymousSuperMixedDirect.event(local0);
                return super.render();
            }
        };
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `AnonymousSuperMixedDirect`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        Base local1 = make();
        java.lang.System.out.println((java.lang.Object) AnonymousSuperMixedDirect.EVENTS);
        java.lang.String local2 = local1.render();
        java.lang.System.out.println(local2);
        java.lang.System.out.println((java.lang.Object) AnonymousSuperMixedDirect.EVENTS);
        return;
    }
}

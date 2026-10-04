// jarde: presentation of `TwoMixedSites` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class TwoMixedSites extends java.lang.Object {
    private static final java.lang.StringBuilder EVENTS = new java.lang.StringBuilder();

    public TwoMixedSites() {
        // @method <init>()V
        // @declaration a constructor of `TwoMixedSites`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static void event(java.lang.String arg0) {
        // @method event(Ljava/lang/String;)V
        // @declaration a static method of `TwoMixedSites`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        if (TwoMixedSites.EVENTS.length() != 0) {
            TwoMixedSites.EVENTS.append('|');
        }
        TwoMixedSites.EVENTS.append(arg0);
        return;
    }

    private static java.lang.String text(java.lang.String arg0, java.lang.String arg1) {
        // @method text(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `TwoMixedSites`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        event("arg:" + arg0);
        return arg1;
    }

    private static int number(java.lang.String arg0, int arg1) {
        // @method number(Ljava/lang/String;I)I
        // @declaration a static method of `TwoMixedSites`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        event("arg:" + arg0);
        return arg1;
    }

    private static java.lang.String capture(java.lang.String arg0) {
        // @method capture(Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `TwoMixedSites`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        event("capture:" + arg0);
        return arg0;
    }

    static Base make1() {
        // @method make1()LBase;
        // @declaration a static method of `TwoMixedSites`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.String local0 = capture("one");
        return new TwoMixedSites$1((java.lang.String) text("a", "x"), number("b", 1), local0);
    }

    static Base make2() {
        // @method make2()LBase;
        // @declaration a static method of `TwoMixedSites`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.String local0 = capture("two");
        return new TwoMixedSites$2((java.lang.String) text("c", "y"), number("d", 2), local0);
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `TwoMixedSites`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        Base local1 = make1();
        Base local2 = make2();
        java.lang.System.out.println((java.lang.Object) TwoMixedSites.EVENTS);
        java.lang.System.out.println((java.lang.String) local1.render());
        java.lang.System.out.println((java.lang.String) local2.render());
        java.lang.System.out.println((java.lang.Object) TwoMixedSites.EVENTS);
        return;
    }
}

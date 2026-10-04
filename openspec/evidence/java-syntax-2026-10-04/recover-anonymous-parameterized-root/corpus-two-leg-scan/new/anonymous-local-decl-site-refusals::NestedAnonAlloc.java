// jarde: presentation of `NestedAnonAlloc` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class NestedAnonAlloc extends java.lang.Object {
    private static final java.lang.StringBuilder EVENTS = new java.lang.StringBuilder();

    public NestedAnonAlloc() {
        // @method <init>()V
        // @declaration a constructor of `NestedAnonAlloc`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static void event(java.lang.String arg0) {
        // @method event(Ljava/lang/String;)V
        // @declaration a static method of `NestedAnonAlloc`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        if (NestedAnonAlloc.EVENTS.length() != 0) {
            NestedAnonAlloc.EVENTS.append('|');
        }
        NestedAnonAlloc.EVENTS.append(arg0);
        return;
    }

    private static java.lang.String text(java.lang.String arg0, java.lang.String arg1) {
        // @method text(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `NestedAnonAlloc`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        event("arg:" + arg0);
        return arg1;
    }

    private static int number(java.lang.String arg0, int arg1) {
        // @method number(Ljava/lang/String;I)I
        // @declaration a static method of `NestedAnonAlloc`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        event("arg:" + arg0);
        return arg1;
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `NestedAnonAlloc`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        NestedAnonAlloc$1 local1 = new NestedAnonAlloc$1((java.lang.String) text("a", "x"), number("b", 1));
        java.lang.System.out.println((java.lang.String) local1.render());
        java.lang.System.out.println((java.lang.Object) NestedAnonAlloc.EVENTS);
        return;
    }
}

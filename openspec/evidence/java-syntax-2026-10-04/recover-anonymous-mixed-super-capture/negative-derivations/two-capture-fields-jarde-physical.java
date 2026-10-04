// jarde: presentation of `TwoCaptureFields` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class TwoCaptureFields extends java.lang.Object {
    private static final java.lang.StringBuilder EVENTS = new java.lang.StringBuilder();

    public TwoCaptureFields() {
        // @method <init>()V
        // @declaration a constructor of `TwoCaptureFields`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static void event(java.lang.String arg0) {
        // @method event(Ljava/lang/String;)V
        // @declaration a static method of `TwoCaptureFields`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        if (TwoCaptureFields.EVENTS.length() != 0) {
            TwoCaptureFields.EVENTS.append('|');
        }
        TwoCaptureFields.EVENTS.append(arg0);
        return;
    }

    private static java.lang.String text(java.lang.String arg0, java.lang.String arg1) {
        // @method text(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `TwoCaptureFields`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        event("arg:" + arg0);
        return arg1;
    }

    private static int number(java.lang.String arg0, int arg1) {
        // @method number(Ljava/lang/String;I)I
        // @declaration a static method of `TwoCaptureFields`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        event("arg:" + arg0);
        return arg1;
    }

    private static java.lang.String capture(java.lang.String arg0) {
        // @method capture(Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `TwoCaptureFields`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        event("capture:" + arg0);
        return arg0;
    }

    static Base make() {
        // @method make()LBase;
        // @declaration a static method of `TwoCaptureFields`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.String local0 = capture("first");
        java.lang.String local1 = capture("second");
        return new TwoCaptureFields$1((java.lang.String) text("super-label", "explicit"), number("super-value", 17), local0, local1);
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `TwoCaptureFields`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        Base local1 = make();
        java.lang.System.out.println((java.lang.Object) TwoCaptureFields.EVENTS);
        java.lang.System.out.println((java.lang.String) local1.render());
        java.lang.System.out.println((java.lang.Object) TwoCaptureFields.EVENTS);
        return;
    }
}

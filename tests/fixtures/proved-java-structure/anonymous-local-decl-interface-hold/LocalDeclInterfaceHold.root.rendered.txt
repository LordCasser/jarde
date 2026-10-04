// jarde: presentation of `LocalDeclInterfaceHold` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class LocalDeclInterfaceHold extends java.lang.Object {
    private static final java.lang.StringBuilder EVENTS = new java.lang.StringBuilder();

    public LocalDeclInterfaceHold() {
        // @method <init>()V
        // @declaration a constructor of `LocalDeclInterfaceHold`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static void event(java.lang.String arg0) {
        // @method event(Ljava/lang/String;)V
        // @declaration a static method of `LocalDeclInterfaceHold`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        if (LocalDeclInterfaceHold.EVENTS.length() != 0) {
            LocalDeclInterfaceHold.EVENTS.append('|');
        }
        LocalDeclInterfaceHold.EVENTS.append(arg0);
        return;
    }

    private static java.lang.String label(java.lang.String arg0) {
        // @method label(Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `LocalDeclInterfaceHold`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        event("label:" + arg0);
        return arg0;
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `LocalDeclInterfaceHold`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        LocalDeclInterfaceHold$1 local1 = new LocalDeclInterfaceHold$1();
        event("after-allocation");
        local1.run();
        java.lang.System.out.println((java.lang.Object) LocalDeclInterfaceHold.EVENTS);
        return;
    }

    static java.lang.String access$000(java.lang.String arg0) {
        // @method access$000(Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `LocalDeclInterfaceHold`, member flags 0x1008
        // recovered from bytecode; presentation is not claimed to compile
        return label(arg0);
    }
}

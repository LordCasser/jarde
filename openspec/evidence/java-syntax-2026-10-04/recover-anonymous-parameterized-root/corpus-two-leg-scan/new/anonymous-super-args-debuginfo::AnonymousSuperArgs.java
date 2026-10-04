// jarde: presentation of `AnonymousSuperArgs` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class AnonymousSuperArgs extends java.lang.Object {
    private static final java.lang.StringBuilder EVENTS = new java.lang.StringBuilder();

    public AnonymousSuperArgs() {
        // @method <init>()V
        // @declaration a constructor of `AnonymousSuperArgs`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static void event(java.lang.String event) {
        // @method event(Ljava/lang/String;)V
        // @declaration a static method of `AnonymousSuperArgs`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        if (AnonymousSuperArgs.EVENTS.length() != 0) {
            AnonymousSuperArgs.EVENTS.append('|');
        }
        AnonymousSuperArgs.EVENTS.append(event);
        return;
    }

    private static java.lang.String text(java.lang.String name, java.lang.String value) {
        // @method text(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `AnonymousSuperArgs`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        event("arg:" + name);
        return value;
    }

    private static int number(java.lang.String name, int value) {
        // @method number(Ljava/lang/String;I)I
        // @declaration a static method of `AnonymousSuperArgs`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        event("arg:" + name);
        return value;
    }

    public static void main(java.lang.String[] args) {
        java.lang.String captured = text("capture", "captured");
        Base instance = new Base((java.lang.String) text("super-label", "explicit"), number("super-value", 17)) {
            java.lang.String render() {
                AnonymousSuperArgs.event((java.lang.String) new java.lang.StringBuilder().append("body:").append(captured).toString());
                return new java.lang.StringBuilder().append((java.lang.String) super.render()).append(":").append(captured).toString();
            }
        };
        java.lang.System.out.println((java.lang.Object) AnonymousSuperArgs.EVENTS);
        java.lang.System.out.println((java.lang.String) instance.render());
        return;
    }
}

// jarde: presentation of `AnonymousTopLevel` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class AnonymousTopLevel extends java.lang.Object {
    private static final java.lang.StringBuilder EVENTS;

    private static int captureCalls;

    private static int chooseCalls;

    static int baseCalls;

    static int renderCalls;

    public AnonymousTopLevel() {
        // @method <init>()V
        // @declaration a constructor of `AnonymousTopLevel`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    private static java.lang.String captureLocal() {
        // @method captureLocal()Ljava/lang/String;
        // @declaration a static method of `AnonymousTopLevel`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        AnonymousTopLevel.captureCalls = AnonymousTopLevel.captureCalls + 1;
        event("capture");
        return "captured";
    }

    private static long choose() {
        // @method choose()J
        // @declaration a static method of `AnonymousTopLevel`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        AnonymousTopLevel.chooseCalls = AnonymousTopLevel.chooseCalls + 1;
        event("choose");
        return 23L;
    }

    static void event(java.lang.String value) {
        // @method event(Ljava/lang/String;)V
        // @declaration a static method of `AnonymousTopLevel`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        if (AnonymousTopLevel.EVENTS.length() > 0) {
            AnonymousTopLevel.EVENTS.append(',');
        }
        AnonymousTopLevel.EVENTS.append(value);
        return;
    }

    static Renderer create() {
        // @method create()LRenderer;
        // @declaration a static method of `AnonymousTopLevel`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.String captured = captureLocal();
        return new AnonymousTopLevel$1(choose(), captured);
    }

    public static void main(java.lang.String[] args) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `AnonymousTopLevel`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        Renderer renderer = create();
        java.lang.System.out.println("value=" + renderer.render());
        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append("events=").append((java.lang.Object) AnonymousTopLevel.EVENTS).toString());
        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append("counts=").append(AnonymousTopLevel.captureCalls).append(",").append(AnonymousTopLevel.chooseCalls).append(",").append(AnonymousTopLevel.baseCalls).append(",").append(AnonymousTopLevel.renderCalls).toString());
        return;
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `AnonymousTopLevel`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        EVENTS = new java.lang.StringBuilder();
    }
}

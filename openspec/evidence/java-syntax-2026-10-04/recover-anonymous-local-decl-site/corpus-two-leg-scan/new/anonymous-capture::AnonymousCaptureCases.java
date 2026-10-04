// jarde: presentation of `AnonymousCaptureCases` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class AnonymousCaptureCases extends java.lang.Object {
    private static final java.lang.StringBuilder EVENTS;

    private static int captureCalls;

    private static int chooseCalls;

    private static int baseCalls;

    private static int renderCalls;

    public AnonymousCaptureCases() {
        // @method <init>()V
        // @declaration a constructor of `AnonymousCaptureCases`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    private static java.lang.String captureLocal() {
        // @method captureLocal()Ljava/lang/String;
        // @declaration a static method of `AnonymousCaptureCases`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        AnonymousCaptureCases.captureCalls = AnonymousCaptureCases.captureCalls + 1;
        event("capture");
        return "captured";
    }

    private static long choose() {
        // @method choose()J
        // @declaration a static method of `AnonymousCaptureCases`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        AnonymousCaptureCases.chooseCalls = AnonymousCaptureCases.chooseCalls + 1;
        event("choose");
        return 7L;
    }

    private static AnonymousCaptureCases$Renderer baseArgumentAndCapture() {
        // @method baseArgumentAndCapture()LAnonymousCaptureCases$Renderer;
        // @declaration a static method of `AnonymousCaptureCases`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.String captured = captureLocal();
        return new AnonymousCaptureCases$1(choose(), captured);
    }

    private static void event(java.lang.String value) {
        // @method event(Ljava/lang/String;)V
        // @declaration a static method of `AnonymousCaptureCases`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        if (AnonymousCaptureCases.EVENTS.length() > 0) {
            AnonymousCaptureCases.EVENTS.append(',');
        }
        AnonymousCaptureCases.EVENTS.append(value);
        return;
    }

    public static void main(java.lang.String[] args) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `AnonymousCaptureCases`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        AnonymousCaptureCases$Renderer baseCase = baseArgumentAndCapture();
        java.lang.System.out.println("base=" + baseCase.render());
        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append("events=").append((java.lang.Object) AnonymousCaptureCases.EVENTS).toString());
        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append("counts=").append(AnonymousCaptureCases.captureCalls).append(",").append(AnonymousCaptureCases.chooseCalls).append(",").append(AnonymousCaptureCases.baseCalls).append(",").append(AnonymousCaptureCases.renderCalls).toString());
        AnonymousCaptureCases$Renderer receiverCase = new AnonymousCaptureCases$Outer(10).captureOther(new AnonymousCaptureCases$Outer(20));
        java.lang.System.out.println("receiver=" + receiverCase.render());
        return;
    }

    static int access$008() {
        // jarde: not recovered: the recovery run for `access$008()I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method access$008()I
        // @declaration a static method of `AnonymousCaptureCases`, member flags 0x1008
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 3 0
        // the instruction at BCI 3 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 6 0
        // the copy at BCI 3 has no proved local assignment
        // @bytecode 9 0
        // the copy at BCI 3 has no proved local assignment
    }

    static void access$100(java.lang.String x0) {
        // @method access$100(Ljava/lang/String;)V
        // @declaration a static method of `AnonymousCaptureCases`, member flags 0x1008
        // recovered from bytecode; presentation is not claimed to compile
        event(x0);
        return;
    }

    static int access$208() {
        // jarde: not recovered: the recovery run for `access$208()I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method access$208()I
        // @declaration a static method of `AnonymousCaptureCases`, member flags 0x1008
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 3 0
        // the instruction at BCI 3 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 6 0
        // the copy at BCI 3 has no proved local assignment
        // @bytecode 9 0
        // the copy at BCI 3 has no proved local assignment
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `AnonymousCaptureCases`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        EVENTS = new java.lang.StringBuilder();
    }
}

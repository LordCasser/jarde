// jarde: presentation of `AnonymousCaptureCases$Outer` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
final class AnonymousCaptureCases$Outer extends java.lang.Object {
    private final int state;

    AnonymousCaptureCases$Outer(int state) {
        // @method <init>(I)V
        // @declaration a constructor of `AnonymousCaptureCases$Outer`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.state = state;
        return;
    }

    AnonymousCaptureCases$Renderer captureOther(AnonymousCaptureCases$Outer other) {
        // @method captureOther(LAnonymousCaptureCases$Outer;)LAnonymousCaptureCases$Renderer;
        // @declaration an instance method of `AnonymousCaptureCases$Outer`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        return new AnonymousCaptureCases$Outer$1(this, other);
    }

    static int access$300(AnonymousCaptureCases$Outer x0) {
        // @method access$300(LAnonymousCaptureCases$Outer;)I
        // @declaration a static method of `AnonymousCaptureCases$Outer`, member flags 0x1008
        // recovered from bytecode; presentation is not claimed to compile
        return x0.state;
    }
}

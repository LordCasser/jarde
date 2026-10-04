// jarde: presentation of `AnonymousCaptureCases$Outer$1` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
class AnonymousCaptureCases$Outer$1 extends java.lang.Object implements AnonymousCaptureCases$Renderer {
    final AnonymousCaptureCases$Outer val$other;

    final AnonymousCaptureCases$Outer this$0;

    // jarde: generic Signature projection refused for `<init>(LAnonymousCaptureCases$Outer;LAnonymousCaptureCases$Outer;)V`: invalid input (jvm_signature_erasure_mismatch): erased Signature disagrees with the physical descriptor at parameter count
    AnonymousCaptureCases$Outer$1(AnonymousCaptureCases$Outer this$0, AnonymousCaptureCases$Outer arg2) {
        // @method <init>(LAnonymousCaptureCases$Outer;LAnonymousCaptureCases$Outer;)V
        // @declaration a constructor of `AnonymousCaptureCases$Outer$1`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.val$other = arg2;
        this.this$0 = this$0;
        return;
    }

    public java.lang.String render() {
        // @method render()Ljava/lang/String;
        // @declaration an instance method of `AnonymousCaptureCases$Outer$1`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return new java.lang.StringBuilder().append(AnonymousCaptureCases$Outer.access$300(this.val$other)).append(":").append(AnonymousCaptureCases$Outer.access$300(this.this$0)).toString();
    }
}

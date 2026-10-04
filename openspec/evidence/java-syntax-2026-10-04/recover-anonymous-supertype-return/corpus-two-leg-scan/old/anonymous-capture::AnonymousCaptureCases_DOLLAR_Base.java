// jarde: presentation of `AnonymousCaptureCases$Base` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
abstract class AnonymousCaptureCases$Base extends java.lang.Object implements AnonymousCaptureCases$Renderer {
    private final long seed;

    AnonymousCaptureCases$Base(long seed) {
        // @method <init>(J)V
        // @declaration a constructor of `AnonymousCaptureCases$Base`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.seed = seed;
        AnonymousCaptureCases.access$008();
        AnonymousCaptureCases.access$100("base(" + seed + ")");
        return;
    }

    long seed() {
        // @method seed()J
        // @declaration an instance method of `AnonymousCaptureCases$Base`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        return this.seed;
    }
}

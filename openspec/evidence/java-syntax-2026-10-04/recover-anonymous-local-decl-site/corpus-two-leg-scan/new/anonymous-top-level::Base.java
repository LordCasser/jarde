// jarde: presentation of `Base` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
abstract class Base extends java.lang.Object implements Renderer {
    private final long seed;

    Base(long seed) {
        // @method <init>(J)V
        // @declaration a constructor of `Base`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.seed = seed;
        AnonymousTopLevel.baseCalls = AnonymousTopLevel.baseCalls + 1;
        AnonymousTopLevel.event("base(" + seed + ")");
        return;
    }

    long seed() {
        // @method seed()J
        // @declaration an instance method of `Base`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        return this.seed;
    }
}

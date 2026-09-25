// jarde: presentation of `VoidBetween` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class VoidBetween extends java.lang.Object {
    public static Target make() {
        // @method make()LTarget;
        // @declaration a static method of `VoidBetween`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        Side.effect();
        return new Target(1);
    }
}

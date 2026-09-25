// jarde: presentation of `RedundantDirectSuperProbe` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class RedundantDirectSuperProbe extends java.lang.Object implements RedundantParent, RedundantChild {
    public RedundantDirectSuperProbe() {
        // @method <init>()V
        // @declaration a constructor of `RedundantDirectSuperProbe`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public int value() {
        // @method value()I
        // @declaration an instance method of `RedundantDirectSuperProbe`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return RedundantParent.super.value();
    }
}

// jarde: presentation of `cf08effects/EffectfulExits` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package cf08effects;

public final class EffectfulExits extends java.lang.Object {
    static int calls;

    public EffectfulExits() {
        // @method <init>()V
        // @declaration a constructor of `cf08effects.EffectfulExits`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static int cost(int arg0) {
        // @method cost(I)I
        // @declaration a static method of `cf08effects.EffectfulExits`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        cf08effects.EffectfulExits.calls = cf08effects.EffectfulExits.calls + 1;
        return arg0;
    }

    public static int pick(int[] arg0) {
        // @method pick([I)I
        // @declaration a static method of `cf08effects.EffectfulExits`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        int local2;
        local2 = 0;
        while (true) {
            if (local2 >= arg0.length) {
                local1 = cost(7);
                break;
            } else {
                local1 = arg0[local2];
                if (local1 == 3) {
                    break;
                } else {
                    local2 = local2 + 1;
                }
            }
        }
        return local1 + cf08effects.EffectfulExits.calls;
    }
}

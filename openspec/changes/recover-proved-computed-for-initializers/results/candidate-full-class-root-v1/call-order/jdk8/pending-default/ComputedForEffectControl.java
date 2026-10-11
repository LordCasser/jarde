// jarde: presentation of `ComputedForEffectControl` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class ComputedForEffectControl extends java.lang.Object {
    static int end;

    static int mode;

    static int fCalls;

    static int gCalls;

    static int order;

    public ComputedForEffectControl() {
        // @method <init>()V
        // @declaration a constructor of `ComputedForEffectControl`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static void reset(int arg0) {
        // @method reset(I)V
        // @declaration a static method of `ComputedForEffectControl`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        ComputedForEffectControl.end = 3;
        ComputedForEffectControl.mode = arg0;
        ComputedForEffectControl.fCalls = 0;
        ComputedForEffectControl.gCalls = 0;
        ComputedForEffectControl.order = 0;
        return;
    }

    static int f() {
        // @method f()I
        // @declaration a static method of `ComputedForEffectControl`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        ComputedForEffectControl.fCalls = ComputedForEffectControl.fCalls + 1;
        ComputedForEffectControl.order = ComputedForEffectControl.order * 10 + 1;
        if (ComputedForEffectControl.mode == 1) {
            throw new java.lang.RuntimeException("fail-f");
        } else {
            return ComputedForEffectControl.end;
        }
    }

    static void g() {
        // @method g()V
        // @declaration a static method of `ComputedForEffectControl`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        ComputedForEffectControl.gCalls = ComputedForEffectControl.gCalls + 1;
        ComputedForEffectControl.order = ComputedForEffectControl.order * 10 + 2;
        ComputedForEffectControl.end = 0;
        if (ComputedForEffectControl.mode == 2) {
            throw new java.lang.RuntimeException("fail-g");
        } else {
            return;
        }
    }

    static int find(int[] arg0, int arg1, int arg2, int arg3, int arg4, int arg5) {
        // @method find([IIIIII)I
        // @declaration a static method of `ComputedForEffectControl`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        int local6;
        int saved0 = f();
        g();
        for (local6 = saved0 - 1; local6 >= arg2; local6 = local6 - 1) {
            if (arg0[local6] == arg1) {
                return local6;
            }
        }
        return -1;
    }
}

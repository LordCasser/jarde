// jarde: presentation of `ShiftEffectOrder` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class ShiftEffectOrder extends java.lang.Object {
    private static int events;

    public ShiftEffectOrder() {
        // @method <init>()V
        // @declaration a constructor of `ShiftEffectOrder`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    private static int step(int arg0, int arg1) {
        // @method step(II)I
        // @declaration a static method of `ShiftEffectOrder`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        ShiftEffectOrder.events = ShiftEffectOrder.events * 10 + arg0;
        return arg1;
    }

    private static int fail(int arg0) {
        // @method fail(I)I
        // @declaration a static method of `ShiftEffectOrder`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        ShiftEffectOrder.events = ShiftEffectOrder.events * 10 + arg0;
        throw new java.lang.IllegalArgumentException("shift operand");
    }

    public static int inline(int arg0) {
        // @method inline(I)I
        // @declaration a static method of `ShiftEffectOrder`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        ShiftEffectOrder.events = 0;
        int local1 = step(1, arg0) << step(2, 2);
        return ShiftEffectOrder.events * 1000 + local1;
    }

    public static int separated(int arg0) {
        // @method separated(I)I
        // @declaration a static method of `ShiftEffectOrder`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        ShiftEffectOrder.events = 0;
        int local1 = step(1, arg0);
        step(3, 0);
        int local2 = local1 << step(2, 2);
        return ShiftEffectOrder.events * 1000 + local2;
    }

    public static int throwLeft(int arg0) {
        // @method throwLeft(I)I
        // @declaration a static method of `ShiftEffectOrder`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        ShiftEffectOrder.events = 0;
        return fail(1) << step(2, arg0);
    }

    public static int throwRight(int arg0) {
        // @method throwRight(I)I
        // @declaration a static method of `ShiftEffectOrder`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        ShiftEffectOrder.events = 0;
        return step(1, arg0) << fail(2);
    }

    public static int events() {
        // @method events()I
        // @declaration a static method of `ShiftEffectOrder`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return ShiftEffectOrder.events;
    }
}

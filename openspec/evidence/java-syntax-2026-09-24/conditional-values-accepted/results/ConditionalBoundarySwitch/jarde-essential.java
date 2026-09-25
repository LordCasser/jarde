// jarde: presentation of `ConditionalBoundarySwitch` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class ConditionalBoundarySwitch extends java.lang.Object {
    private static int trace;

    public ConditionalBoundarySwitch() {
        // @method <init>()V
        // @declaration a constructor of `ConditionalBoundarySwitch`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    private static int left() {
        // @method left()I
        // @declaration a static method of `ConditionalBoundarySwitch`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        ConditionalBoundarySwitch.trace = ConditionalBoundarySwitch.trace * 10 + 1;
        return 101;
    }

    private static int middle() {
        // @method middle()I
        // @declaration a static method of `ConditionalBoundarySwitch`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        ConditionalBoundarySwitch.trace = ConditionalBoundarySwitch.trace * 10 + 2;
        return 202;
    }

    private static int fallback() {
        // @method fallback()I
        // @declaration a static method of `ConditionalBoundarySwitch`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        ConditionalBoundarySwitch.trace = ConditionalBoundarySwitch.trace * 10 + 3;
        return 303;
    }

    public static void reset() {
        // @method reset()V
        // @declaration a static method of `ConditionalBoundarySwitch`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        ConditionalBoundarySwitch.trace = 0;
        return;
    }

    public static int trace() {
        // @method trace()I
        // @declaration a static method of `ConditionalBoundarySwitch`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return ConditionalBoundarySwitch.trace;
    }

    public static int choose(int arg0) {
        // @method choose(I)I
        // @declaration a static method of `ConditionalBoundarySwitch`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        switch (arg0) {
            case 0:
                local1 = left();
                break;
            case 1:
                local1 = middle();
                break;
            default:
                local1 = fallback();
                break;
        }
        return local1;
    }
}

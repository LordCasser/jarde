// jarde: presentation of `RawIterableIterator` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class RawIterableIterator extends java.lang.Object {
    public static int touches;

    public RawIterableIterator() {
        // @method <init>()V
        // @declaration a constructor of `RawIterableIterator`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static int rawCast(java.lang.Iterable arg0) {
        // @method rawCast(Ljava/lang/Iterable;)I
        // @declaration a static method of `RawIterableIterator`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.util.Iterator local1;
        int local2;
        local1 = arg0.iterator();
        local2 = 0;
        while (local1.hasNext()) {
            java.lang.Object local3 = local1.next();
            java.lang.String local4 = (java.lang.String) local3;
            local2 = local2 + local4.length();
        }
        return local2;
    }

    public static int nextTwiceCastBeforeTouch(java.lang.Iterable arg0) {
        // @method nextTwiceCastBeforeTouch(Ljava/lang/Iterable;)I
        // @declaration a static method of `RawIterableIterator`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.util.Iterator local1;
        int local2;
        local1 = arg0.iterator();
        local2 = 0;
        while (local1.hasNext()) {
            java.lang.Object local3 = local1.next();
            java.lang.String local4 = (java.lang.String) local3;
            RawIterableIterator.touches = RawIterableIterator.touches + 1;
            local2 = local2 + (local4.length() + local3.toString().length());
        }
        return local2;
    }

    public static int nextTwiceTouchBeforeCast(java.lang.Iterable arg0) {
        // @method nextTwiceTouchBeforeCast(Ljava/lang/Iterable;)I
        // @declaration a static method of `RawIterableIterator`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.util.Iterator local1;
        int local2;
        local1 = arg0.iterator();
        local2 = 0;
        while (local1.hasNext()) {
            java.lang.Object local3 = local1.next();
            RawIterableIterator.touches = RawIterableIterator.touches + 1;
            java.lang.String local4 = (java.lang.String) local3;
            local2 = local2 + (local4.length() + local3.toString().length());
        }
        return local2;
    }

    public static int touchBeforeNext(java.lang.Iterable arg0) {
        // @method touchBeforeNext(Ljava/lang/Iterable;)I
        // @declaration a static method of `RawIterableIterator`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.util.Iterator local1;
        int local2;
        local1 = arg0.iterator();
        local2 = 0;
        while (local1.hasNext()) {
            RawIterableIterator.touches = RawIterableIterator.touches + 1;
            java.lang.Object local3 = local1.next();
            java.lang.String local4 = (java.lang.String) local3;
            local2 = local2 + local4.length();
        }
        return local2;
    }
}

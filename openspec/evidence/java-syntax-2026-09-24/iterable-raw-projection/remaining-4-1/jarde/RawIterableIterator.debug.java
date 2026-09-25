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

    public static int rawCast(java.lang.Iterable values) {
        // @method rawCast(Ljava/lang/Iterable;)I
        // @declaration a static method of `RawIterableIterator`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.util.Iterator it;
        int result;
        it = values.iterator();
        result = 0;
        while (it.hasNext()) {
            java.lang.Object item = it.next();
            java.lang.String text = (java.lang.String) item;
            result = result + text.length();
        }
        return result;
    }

    public static int nextTwiceCastBeforeTouch(java.lang.Iterable values) {
        // @method nextTwiceCastBeforeTouch(Ljava/lang/Iterable;)I
        // @declaration a static method of `RawIterableIterator`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.util.Iterator it;
        int result;
        it = values.iterator();
        result = 0;
        while (it.hasNext()) {
            java.lang.Object item = it.next();
            java.lang.String text = (java.lang.String) item;
            RawIterableIterator.touches = RawIterableIterator.touches + 1;
            result = result + (text.length() + item.toString().length());
        }
        return result;
    }

    public static int nextTwiceTouchBeforeCast(java.lang.Iterable values) {
        // @method nextTwiceTouchBeforeCast(Ljava/lang/Iterable;)I
        // @declaration a static method of `RawIterableIterator`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.util.Iterator it;
        int result;
        it = values.iterator();
        result = 0;
        while (it.hasNext()) {
            java.lang.Object item = it.next();
            RawIterableIterator.touches = RawIterableIterator.touches + 1;
            java.lang.String text = (java.lang.String) item;
            result = result + (text.length() + item.toString().length());
        }
        return result;
    }

    public static int touchBeforeNext(java.lang.Iterable values) {
        // @method touchBeforeNext(Ljava/lang/Iterable;)I
        // @declaration a static method of `RawIterableIterator`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.util.Iterator it;
        int result;
        it = values.iterator();
        result = 0;
        while (it.hasNext()) {
            RawIterableIterator.touches = RawIterableIterator.touches + 1;
            java.lang.Object item = it.next();
            java.lang.String text = (java.lang.String) item;
            result = result + text.length();
        }
        return result;
    }
}

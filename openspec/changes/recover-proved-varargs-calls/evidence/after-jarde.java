// jarde: presentation of `VarargsCalls` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class VarargsCalls extends java.lang.Object {
    static int effects;

    static java.lang.String order;

    public VarargsCalls() {
        // @method <init>()V
        // @declaration a constructor of `VarargsCalls`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static int mark(int arg0) {
        // @method mark(I)I
        // @declaration a static method of `VarargsCalls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        VarargsCalls.effects = VarargsCalls.effects + 1;
        VarargsCalls.order = new java.lang.StringBuilder().append(VarargsCalls.order).append(arg0).toString();
        if (arg0 == 9) {
            throw new java.lang.IllegalStateException("stop");
        } else {
            return arg0;
        }
    }

    static int count(int... arg0) {
        // @method count([I)I
        // @declaration a static method of `VarargsCalls`, member flags 0x0088
        // recovered from bytecode; presentation is not claimed to compile
        return arg0.length;
    }

    static int objects(java.lang.Object... arg0) {
        // @method objects([Ljava/lang/Object;)I
        // @declaration a static method of `VarargsCalls`, member flags 0x0088
        // recovered from bytecode; presentation is not claimed to compile
        return arg0.length;
    }

    static java.lang.String shape(java.lang.Object... arg0) {
        // @method shape([Ljava/lang/Object;)Ljava/lang/String;
        // @declaration a static method of `VarargsCalls`, member flags 0x0088
        // recovered from bytecode; presentation is not claimed to compile
        return new java.lang.StringBuilder().append((java.lang.String) arg0.getClass().getName()).append("/").append(arg0.length).toString();
    }

    static int strings(java.lang.String... arg0) {
        // @method strings([Ljava/lang/String;)I
        // @declaration a static method of `VarargsCalls`, member flags 0x0088
        // recovered from bytecode; presentation is not claimed to compile
        return arg0.length;
    }

    static int plain(int[] arg0) {
        // @method plain([I)I
        // @declaration a static method of `VarargsCalls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return arg0.length;
    }

    static int overloaded(java.lang.Object... arg0) {
        // @method overloaded([Ljava/lang/Object;)I
        // @declaration a static method of `VarargsCalls`, member flags 0x0088
        // recovered from bytecode; presentation is not claimed to compile
        return arg0.length;
    }

    static int overloaded(java.lang.String arg0) {
        // @method overloaded(Ljava/lang/String;)I
        // @declaration a static method of `VarargsCalls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return arg0.length();
    }

    static int ordered() {
        // @method ordered()I
        // @declaration a static method of `VarargsCalls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return count(mark(1), mark(2), mark(3));
    }

    static int oneString() {
        // @method oneString()I
        // @declaration a static method of `VarargsCalls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return strings("one");
    }

    static int objectValues() {
        // @method objectValues()I
        // @declaration a static method of `VarargsCalls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return objects("a", "b");
    }

    static int explicitVarargsArray() {
        // @method explicitVarargsArray()I
        // @declaration a static method of `VarargsCalls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return count(mark(4), mark(5));
    }

    static int ordinaryArray() {
        // @method ordinaryArray()I
        // @declaration a static method of `VarargsCalls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return plain(new int[]{mark(6), mark(7)});
    }

    static int heldArray() {
        // @method heldArray()I
        // @declaration a static method of `VarargsCalls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        int[] local0 = new int[]{mark(8), mark(1)};
        return count(local0);
    }

    static int overloadedArray() {
        // @method overloadedArray()I
        // @declaration a static method of `VarargsCalls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return overloaded(new java.lang.Object[]{"x"});
    }

    static java.lang.String nullElement() {
        // @method nullElement()Ljava/lang/String;
        // @declaration a static method of `VarargsCalls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return shape(new java.lang.Object[]{null});
    }

    static java.lang.String arrayElement() {
        // @method arrayElement()Ljava/lang/String;
        // @declaration a static method of `VarargsCalls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return shape((java.lang.Object[]) new java.lang.String[]{"x"});
    }

    static int exceptionOrder() {
        // @method exceptionOrder()I
        // @declaration a static method of `VarargsCalls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return count(mark(1), mark(9), mark(2));
    }
}

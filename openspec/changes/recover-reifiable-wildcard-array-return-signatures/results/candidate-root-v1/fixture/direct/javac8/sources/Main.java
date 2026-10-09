// jarde: presentation of `Main` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class Main extends java.lang.Object {
    private static int trace;

    public Main() {
        // @method <init>()V
        // @declaration a constructor of `Main`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    private static int mark(int arg0) {
        // @method mark(I)I
        // @declaration a static method of `Main`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        Main.trace = Main.trace * 10 + arg0;
        return arg0;
    }

    private static java.lang.Byte byteValue(int arg0) {
        // @method byteValue(I)Ljava/lang/Byte;
        // @declaration a static method of `Main`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        return java.lang.Byte.valueOf((byte) arg0);
    }

    private static java.lang.Short shortValue(int arg0) {
        // @method shortValue(I)Ljava/lang/Short;
        // @declaration a static method of `Main`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        return java.lang.Short.valueOf((short) arg0);
    }

    private static java.lang.Integer integerValue(int arg0) {
        // @method integerValue(I)Ljava/lang/Integer;
        // @declaration a static method of `Main`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        return java.lang.Integer.valueOf(arg0);
    }

    private static java.lang.Long longValue(int arg0) {
        // @method longValue(I)Ljava/lang/Long;
        // @declaration a static method of `Main`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        return java.lang.Long.valueOf((long) arg0);
    }

    private static java.lang.Float floatValue(int arg0) {
        // @method floatValue(I)Ljava/lang/Float;
        // @declaration a static method of `Main`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        return java.lang.Float.valueOf((float) arg0);
    }

    private static java.lang.Double doubleValue(int arg0) {
        // @method doubleValue(I)Ljava/lang/Double;
        // @declaration a static method of `Main`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        return java.lang.Double.valueOf((double) arg0);
    }

    private static java.lang.Number numberValue(int arg0) {
        // @method numberValue(I)Ljava/lang/Number;
        // @declaration a static method of `Main`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        return java.lang.Integer.valueOf(arg0);
    }

    private static java.lang.Object objectValue(int arg0) {
        // @method objectValue(I)Ljava/lang/Object;
        // @declaration a static method of `Main`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        return java.lang.String.valueOf(arg0);
    }

    public static java.lang.Number[] exactNumber() {
        // @method exactNumber()[Ljava/lang/Number;
        // @declaration a static method of `Main`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return new java.lang.Number[]{numberValue(mark(1))};
    }

    public static java.lang.Object[] objectElement() {
        // @method objectElement()[Ljava/lang/Object;
        // @declaration a static method of `Main`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return new java.lang.Object[]{objectValue(mark(1))};
    }

    public static java.lang.Object[] nullElement() {
        // @method nullElement()[Ljava/lang/Object;
        // @declaration a static method of `Main`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return new java.lang.Object[]{null};
    }

    private static java.lang.String stringValue(int arg0) {
        // @method stringValue(I)Ljava/lang/String;
        // @declaration a static method of `Main`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        return java.lang.String.valueOf(arg0);
    }

    private static java.lang.StringBuilder builderValue(int arg0) {
        // @method builderValue(I)Ljava/lang/StringBuilder;
        // @declaration a static method of `Main`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        return new java.lang.StringBuilder((java.lang.String) java.lang.String.valueOf(arg0));
    }

    // jarde: generic Signature projection refused for `listValue(I)Ljava/util/ArrayList;`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types
    private static java.util.ArrayList listValue(int arg0) {
        // @method listValue(I)Ljava/util/ArrayList;
        // @declaration a static method of `Main`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        return new java.util.ArrayList((java.util.Collection) java.util.Collections.singletonList((java.lang.Object) java.lang.Integer.valueOf(mark(arg0))));
    }

    // jarde: generic Signature projection refused for `setValue(I)Ljava/util/HashSet;`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types
    private static java.util.HashSet setValue(int arg0) {
        // @method setValue(I)Ljava/util/HashSet;
        // @declaration a static method of `Main`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        return new java.util.HashSet((java.util.Collection) java.util.Collections.singletonList((java.lang.Object) java.lang.Integer.valueOf(mark(arg0))));
    }

    private static java.lang.IllegalStateException stateFailure(int arg0) {
        // @method stateFailure(I)Ljava/lang/IllegalStateException;
        // @declaration a static method of `Main`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        return new java.lang.IllegalStateException((java.lang.String) java.lang.String.valueOf(mark(arg0)));
    }

    private static java.lang.IllegalArgumentException argumentFailure(int arg0) {
        // @method argumentFailure(I)Ljava/lang/IllegalArgumentException;
        // @declaration a static method of `Main`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        return new java.lang.IllegalArgumentException((java.lang.String) java.lang.String.valueOf(mark(arg0)));
    }

    private static java.lang.Integer[] integerArray(int arg0) {
        // @method integerArray(I)[Ljava/lang/Integer;
        // @declaration a static method of `Main`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        return new java.lang.Integer[]{java.lang.Integer.valueOf(arg0)};
    }

    private static java.lang.Long[] longArray(int arg0) {
        // @method longArray(I)[Ljava/lang/Long;
        // @declaration a static method of `Main`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        return new java.lang.Long[]{java.lang.Long.valueOf((long) arg0)};
    }

    // jarde: generic Signature projection refused for `listArray(I)[Ljava/util/ArrayList;`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types
    private static java.util.ArrayList[] listArray(int arg0) {
        // @method listArray(I)[Ljava/util/ArrayList;
        // @declaration a static method of `Main`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        return new java.util.ArrayList[]{listValue(arg0)};
    }

    // jarde: generic Signature projection refused for `setArray(I)[Ljava/util/HashSet;`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types
    private static java.util.HashSet[] setArray(int arg0) {
        // @method setArray(I)[Ljava/util/HashSet;
        // @declaration a static method of `Main`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        return new java.util.HashSet[]{setValue(arg0)};
    }

    private static DerivedA derivedA(int arg0) {
        // @method derivedA(I)LDerivedA;
        // @declaration a static method of `Main`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        return new DerivedA(arg0);
    }

    private static DerivedB derivedB(int arg0) {
        // @method derivedB(I)LDerivedB;
        // @declaration a static method of `Main`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        return new DerivedB(arg0);
    }

    private static DerivedA[] derivedAArray(int arg0) {
        // @method derivedAArray(I)[LDerivedA;
        // @declaration a static method of `Main`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        return new DerivedA[]{derivedA(arg0)};
    }

    private static DerivedB[] derivedBArray(int arg0) {
        // @method derivedBArray(I)[LDerivedB;
        // @declaration a static method of `Main`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        return new DerivedB[]{derivedB(arg0)};
    }

    public static java.lang.Number[] boxedDirect() {
        // @method boxedDirect()[Ljava/lang/Number;
        // @declaration a static method of `Main`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return new java.lang.Number[]{new java.lang.Byte((byte) mark(1)), new java.lang.Short((short) mark(2)), new java.lang.Integer(mark(3)), new java.lang.Long((long) mark(4)), new java.lang.Float((float) mark(5)), new java.lang.Double((double) mark(6))};
    }

    public static java.lang.CharSequence[] sequenceDirect() {
        // @method sequenceDirect()[Ljava/lang/CharSequence;
        // @declaration a static method of `Main`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return new java.lang.CharSequence[]{new java.lang.String((java.lang.String) java.lang.String.valueOf(mark(1))), new java.lang.StringBuilder((java.lang.String) java.lang.String.valueOf(mark(2)))};
    }

    public static java.util.Collection<?>[] collectionDirect() {
        // jarde: generic Signature `()[Ljava/util/Collection<*>;` projected after descriptor erasure and same-run AST/SSA parameter-return proof; same-class call binding proved
        // @method collectionDirect()[Ljava/util/Collection;
        // @declaration a static method of `Main`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return new java.util.Collection[]{new java.util.ArrayList((java.util.Collection) java.util.Collections.singletonList((java.lang.Object) java.lang.Integer.valueOf(mark(1)))), new java.util.HashSet((java.util.Collection) java.util.Collections.singletonList((java.lang.Object) java.lang.Integer.valueOf(mark(2))))};
    }

    public static java.lang.Throwable[] throwableDirect() {
        // @method throwableDirect()[Ljava/lang/Throwable;
        // @declaration a static method of `Main`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return new java.lang.Throwable[]{new java.lang.IllegalStateException((java.lang.String) java.lang.String.valueOf(mark(1))), new java.lang.IllegalArgumentException((java.lang.String) java.lang.String.valueOf(mark(2)))};
    }

    public static java.lang.Number[][] numberGridDirect() {
        // @method numberGridDirect()[[Ljava/lang/Number;
        // @declaration a static method of `Main`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return new java.lang.Number[][]{new java.lang.Integer[]{java.lang.Integer.valueOf(mark(1))}, new java.lang.Long[]{java.lang.Long.valueOf((long) mark(2))}};
    }

    public static java.util.Collection<?>[][] collectionGridDirect() {
        // jarde: generic Signature `()[[Ljava/util/Collection<*>;` projected after descriptor erasure and same-run AST/SSA parameter-return proof; same-class call binding proved
        // @method collectionGridDirect()[[Ljava/util/Collection;
        // @declaration a static method of `Main`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return new java.util.Collection[][]{new java.util.ArrayList[]{listValue(mark(1))}, new java.util.HashSet[]{setValue(mark(2))}};
    }

    public static Base[] ownTwoHopDirect() {
        // @method ownTwoHopDirect()[LBase;
        // @declaration a static method of `Main`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return new Base[]{new DerivedA(mark(1)), new DerivedB(mark(2))};
    }

    public static LocalInterface[] ownInterfaceDirect() {
        // @method ownInterfaceDirect()[LLocalInterface;
        // @declaration a static method of `Main`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return new LocalInterface[]{new DerivedA(mark(1)), new DerivedB(mark(2))};
    }

    public static Base[][] ownGridDirect() {
        // @method ownGridDirect()[[LBase;
        // @declaration a static method of `Main`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return new Base[][]{new DerivedA[]{new DerivedA(mark(1))}, new DerivedB[]{new DerivedB(mark(2))}};
    }

    private static void observe(java.lang.Object[] arg0) {
        // @method observe([Ljava/lang/Object;)V
        // @declaration a static method of `Main`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.Object local1 = arg0[0];
        if (local1 == null) {
            java.lang.System.out.println("null");
        } else {
            java.lang.System.out.println((java.lang.String) local1.getClass().getName());
        }
        java.lang.System.out.println(Main.trace);
        return;
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `Main`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        Main.trace = 0;
        observe((java.lang.Object[]) boxedDirect());
        Main.trace = 0;
        observe((java.lang.Object[]) sequenceDirect());
        Main.trace = 0;
        observe((java.lang.Object[]) collectionDirect());
        Main.trace = 0;
        observe((java.lang.Object[]) throwableDirect());
        Main.trace = 0;
        observe((java.lang.Object[]) numberGridDirect());
        Main.trace = 0;
        observe((java.lang.Object[]) collectionGridDirect());
        Main.trace = 0;
        observe((java.lang.Object[]) ownTwoHopDirect());
        Main.trace = 0;
        observe((java.lang.Object[]) ownInterfaceDirect());
        Main.trace = 0;
        observe((java.lang.Object[]) ownGridDirect());
        Main.trace = 0;
        observe((java.lang.Object[]) exactNumber());
        Main.trace = 0;
        observe((java.lang.Object[]) objectElement());
        Main.trace = 0;
        observe((java.lang.Object[]) nullElement());
        return;
    }
}

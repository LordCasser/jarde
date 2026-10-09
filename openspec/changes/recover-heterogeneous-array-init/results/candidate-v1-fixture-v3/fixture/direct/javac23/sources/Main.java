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
        // jarde: not recovered: the recovery run for `boxedDirect()[Ljava/lang/Number;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method boxedDirect()[Ljava/lang/Number;
        // @declaration a static method of `Main`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 2
        // the array instruction at BCI 2 produces a value nothing in this body reads, so the instruction the bytecode runs has no place in the text
        // @bytecode 5 2
        // the instruction at BCI 5 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 7
        // the instruction at BCI 7 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 10
        // the instruction at BCI 10 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 19 2 16 15 12
        // the copy at BCI 5 has no proved local assignment
        // @bytecode 20 2
        // the instruction at BCI 20 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 22
        // the instruction at BCI 22 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 25
        // the instruction at BCI 25 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 34 2 31 30 27
        // the copy at BCI 20 has no proved local assignment
        // @bytecode 35 2
        // the instruction at BCI 35 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 37
        // the instruction at BCI 37 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 40
        // the instruction at BCI 40 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 48 2 45 42
        // the copy at BCI 35 has no proved local assignment
        // @bytecode 49 2
        // the instruction at BCI 49 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 51
        // the instruction at BCI 51 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 54
        // the instruction at BCI 54 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 63 2 60 59 56
        // the copy at BCI 49 has no proved local assignment
        // @bytecode 64 2
        // the instruction at BCI 64 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 66
        // the instruction at BCI 66 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 69
        // the instruction at BCI 69 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 78 2 75 74 71
        // the copy at BCI 64 has no proved local assignment
        // @bytecode 79 2
        // the instruction at BCI 79 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 81
        // the instruction at BCI 81 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 84
        // the instruction at BCI 84 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 94 2 91 90 87
        // the copy at BCI 79 has no proved local assignment
        // @bytecode 95 2
        // the copy at BCI 79 has no proved local assignment
    }

    public static java.lang.CharSequence[] sequenceDirect() {
        // jarde: not recovered: the recovery run for `sequenceDirect()[Ljava/lang/CharSequence;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method sequenceDirect()[Ljava/lang/CharSequence;
        // @declaration a static method of `Main`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 1
        // the array instruction at BCI 1 produces a value nothing in this body reads, so the instruction the bytecode runs has no place in the text
        // @bytecode 4 1
        // the instruction at BCI 4 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 6
        // the instruction at BCI 6 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 9
        // the instruction at BCI 9 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 20 1 17 14 11
        // the copy at BCI 4 has no proved local assignment
        // @bytecode 21 1
        // the instruction at BCI 21 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 23
        // the instruction at BCI 23 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 26
        // the instruction at BCI 26 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 37 1 34 31 28
        // the copy at BCI 21 has no proved local assignment
        // @bytecode 38 1
        // the copy at BCI 21 has no proved local assignment
    }

    // jarde: generic Signature projection refused for `collectionDirect()[Ljava/util/Collection;`: unsupported (ordinary_generic_source_unproved): method body or no-body declaration has no complete source proof
    public static java.util.Collection[] collectionDirect() {
        // jarde: not recovered: the recovery run for `collectionDirect()[Ljava/util/Collection;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method collectionDirect()[Ljava/util/Collection;
        // @declaration a static method of `Main`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 1
        // the array instruction at BCI 1 produces a value nothing in this body reads, so the instruction the bytecode runs has no place in the text
        // @bytecode 4 1
        // the instruction at BCI 4 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 6
        // the instruction at BCI 6 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 9
        // the instruction at BCI 9 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 23 1 20 17 14 11
        // the copy at BCI 4 has no proved local assignment
        // @bytecode 24 1
        // the instruction at BCI 24 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 26
        // the instruction at BCI 26 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 29
        // the instruction at BCI 29 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 43 1 40 37 34 31
        // the copy at BCI 24 has no proved local assignment
        // @bytecode 44 1
        // the copy at BCI 24 has no proved local assignment
    }

    public static java.lang.Throwable[] throwableDirect() {
        // jarde: not recovered: the recovery run for `throwableDirect()[Ljava/lang/Throwable;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method throwableDirect()[Ljava/lang/Throwable;
        // @declaration a static method of `Main`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 1
        // the array instruction at BCI 1 produces a value nothing in this body reads, so the instruction the bytecode runs has no place in the text
        // @bytecode 4 1
        // the instruction at BCI 4 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 6
        // the instruction at BCI 6 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 9
        // the instruction at BCI 9 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 20 1 17 14 11
        // the copy at BCI 4 has no proved local assignment
        // @bytecode 21 1
        // the instruction at BCI 21 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 23
        // the instruction at BCI 23 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 26
        // the instruction at BCI 26 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 37 1 34 31 28
        // the copy at BCI 21 has no proved local assignment
        // @bytecode 38 1
        // the copy at BCI 21 has no proved local assignment
    }

    public static java.lang.Number[][] numberGridDirect() {
        // jarde: not recovered: the recovery run for `numberGridDirect()[[Ljava/lang/Number;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method numberGridDirect()[[Ljava/lang/Number;
        // @declaration a static method of `Main`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 1
        // the array instruction at BCI 1 produces a value nothing in this body reads, so the instruction the bytecode runs has no place in the text
        // @bytecode 4 1
        // the instruction at BCI 4 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 7
        // the array instruction at BCI 7 produces a value nothing in this body reads, so the instruction the bytecode runs has no place in the text
        // @bytecode 10 7
        // the instruction at BCI 10 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 19 7 16 13
        // the copy at BCI 10 has no proved local assignment
        // @bytecode 20 1 7
        // the copy at BCI 4 has no proved local assignment
        // @bytecode 21 1
        // the instruction at BCI 21 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 24
        // the array instruction at BCI 24 produces a value nothing in this body reads, so the instruction the bytecode runs has no place in the text
        // @bytecode 27 24
        // the instruction at BCI 27 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 37 24 34 33 30
        // the copy at BCI 27 has no proved local assignment
        // @bytecode 38 1 24
        // the copy at BCI 21 has no proved local assignment
        // @bytecode 39 1
        // the copy at BCI 21 has no proved local assignment
    }

    // jarde: generic Signature projection refused for `collectionGridDirect()[[Ljava/util/Collection;`: unsupported (ordinary_generic_source_unproved): method body or no-body declaration has no complete source proof
    public static java.util.Collection[][] collectionGridDirect() {
        // jarde: not recovered: the recovery run for `collectionGridDirect()[[Ljava/util/Collection;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method collectionGridDirect()[[Ljava/util/Collection;
        // @declaration a static method of `Main`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 1
        // the array instruction at BCI 1 produces a value nothing in this body reads, so the instruction the bytecode runs has no place in the text
        // @bytecode 4 1
        // the instruction at BCI 4 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 7
        // the array instruction at BCI 7 produces a value nothing in this body reads, so the instruction the bytecode runs has no place in the text
        // @bytecode 10 7
        // the instruction at BCI 10 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 19 7 16 13
        // the copy at BCI 10 has no proved local assignment
        // @bytecode 20 1 7
        // the copy at BCI 4 has no proved local assignment
        // @bytecode 21 1
        // the instruction at BCI 21 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 24
        // the array instruction at BCI 24 produces a value nothing in this body reads, so the instruction the bytecode runs has no place in the text
        // @bytecode 27 24
        // the instruction at BCI 27 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 36 24 33 30
        // the copy at BCI 27 has no proved local assignment
        // @bytecode 37 1 24
        // the copy at BCI 21 has no proved local assignment
        // @bytecode 38 1
        // the copy at BCI 21 has no proved local assignment
    }

    public static Base[] ownTwoHopDirect() {
        // jarde: not recovered: the recovery run for `ownTwoHopDirect()[LBase;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method ownTwoHopDirect()[LBase;
        // @declaration a static method of `Main`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 1
        // the array instruction at BCI 1 produces a value nothing in this body reads, so the instruction the bytecode runs has no place in the text
        // @bytecode 4 1
        // the instruction at BCI 4 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 6
        // the instruction at BCI 6 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 9
        // the instruction at BCI 9 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 17 1 14 11
        // the copy at BCI 4 has no proved local assignment
        // @bytecode 18 1
        // the instruction at BCI 18 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 20
        // the instruction at BCI 20 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 23
        // the instruction at BCI 23 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 31 1 28 25
        // the copy at BCI 18 has no proved local assignment
        // @bytecode 32 1
        // the copy at BCI 18 has no proved local assignment
    }

    public static LocalInterface[] ownInterfaceDirect() {
        // jarde: not recovered: the recovery run for `ownInterfaceDirect()[LLocalInterface;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method ownInterfaceDirect()[LLocalInterface;
        // @declaration a static method of `Main`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 1
        // the array instruction at BCI 1 produces a value nothing in this body reads, so the instruction the bytecode runs has no place in the text
        // @bytecode 4 1
        // the instruction at BCI 4 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 6
        // the instruction at BCI 6 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 9
        // the instruction at BCI 9 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 17 1 14 11
        // the copy at BCI 4 has no proved local assignment
        // @bytecode 18 1
        // the instruction at BCI 18 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 20
        // the instruction at BCI 20 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 23
        // the instruction at BCI 23 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 31 1 28 25
        // the copy at BCI 18 has no proved local assignment
        // @bytecode 32 1
        // the copy at BCI 18 has no proved local assignment
    }

    public static Base[][] ownGridDirect() {
        // jarde: not recovered: the recovery run for `ownGridDirect()[[LBase;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method ownGridDirect()[[LBase;
        // @declaration a static method of `Main`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 1
        // the array instruction at BCI 1 produces a value nothing in this body reads, so the instruction the bytecode runs has no place in the text
        // @bytecode 4 1
        // the instruction at BCI 4 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 7
        // the array instruction at BCI 7 produces a value nothing in this body reads, so the instruction the bytecode runs has no place in the text
        // @bytecode 10 7
        // the instruction at BCI 10 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 12
        // the instruction at BCI 12 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 15
        // the instruction at BCI 15 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 23 7 20 17
        // the copy at BCI 10 has no proved local assignment
        // @bytecode 24 1 7
        // the copy at BCI 4 has no proved local assignment
        // @bytecode 25 1
        // the instruction at BCI 25 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 28
        // the array instruction at BCI 28 produces a value nothing in this body reads, so the instruction the bytecode runs has no place in the text
        // @bytecode 31 28
        // the instruction at BCI 31 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 33
        // the instruction at BCI 33 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 36
        // the instruction at BCI 36 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 44 28 41 38
        // the copy at BCI 31 has no proved local assignment
        // @bytecode 45 1 28
        // the copy at BCI 25 has no proved local assignment
        // @bytecode 46 1
        // the copy at BCI 25 has no proved local assignment
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

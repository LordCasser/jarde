// jarde: presentation of `TypeUseRunner` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
final class TypeUseRunner extends java.lang.Object {
    TypeUseRunner() {
        // @method <init>()V
        // @declaration a constructor of `TypeUseRunner`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    private static java.lang.String value(java.lang.reflect.AnnotatedType arg0) {
        // @method value(Ljava/lang/reflect/AnnotatedType;)Ljava/lang/String;
        // @declaration a static method of `TypeUseRunner`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        TypeMark local1 = (TypeMark) arg0.getAnnotation(TypeMark.class);
        if (local1 == null) {
        } else {
            // @bytecode 22
            // the saved producer at BCI 22 has no bounded final expression consumer
        }
        // @bytecode 27
        // the value at BCI 27 is the entry state of stack depth 0, which no instruction produced
    }

    public static void main(java.lang.String[] arg0) throws java.lang.Exception {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `TypeUseRunner`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.reflect.Field local1 = TypeUseSubject.class.getDeclaredField("field");
        java.lang.reflect.Method local2 = TypeUseSubject.class.getDeclaredMethod("value", new java.lang.Class[0]);
        // @bytecode 25
        // the array instruction at BCI 25 produces a value nothing in this body reads, so the instruction the bytecode runs has no place in the text
        // @bytecode 28 25
        // the instruction at BCI 28 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 32 25 30
        // the value at BCI 32 comes from an Duplicate at BCI 28, which produces no expression this subset writes
        // @bytecode 36 33 20 25
        // the value at BCI 36 comes from an Duplicate at BCI 28, which produces no expression this subset writes
        java.lang.System.out.println((java.lang.String) value((java.lang.reflect.AnnotatedType) local1.getAnnotatedType()));
        java.lang.System.out.println((java.lang.String) value((java.lang.reflect.AnnotatedType) local2.getAnnotatedReturnType()));
        // @bytecode 63 66 67 70 71 72 75
        // the statement at BCI 75 reads `local3`, and no statement of this body declared that local: the write that would have declared it was refused, so its name cannot be read here (P3 2b.2)
        java.lang.System.out.println((java.lang.String) java.util.Arrays.toString((char[]) new TypeUseSubject().echo("ok").toCharArray()));
        return;
    }
}

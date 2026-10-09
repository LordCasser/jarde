// jarde: presentation of `Main` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class Main extends java.lang.Object {
    public Main() {
        // @method <init>()V
        // @declaration a constructor of `Main`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static void event(java.lang.String arg0, java.lang.String arg1) {
        // @method event(Ljava/lang/String;Ljava/lang/String;)V
        // @declaration a static method of `Main`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.print(arg0);
        java.lang.System.out.print(':');
        java.lang.System.out.println(arg1);
        return;
    }

    public static java.lang.String mark(java.lang.String arg0) {
        // @method mark(Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `Main`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        event("mark", arg0);
        return arg0;
    }

    public static java.lang.CharSequence[] sequence() {
        // jarde: not recovered: the recovery run for `sequence()[Ljava/lang/CharSequence;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method sequence()[Ljava/lang/CharSequence;
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
        // @bytecode 18 1 15 12
        // the copy at BCI 4 has no proved local assignment
        // @bytecode 19 1
        // the instruction at BCI 19 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 21
        // the instruction at BCI 21 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 24
        // the instruction at BCI 24 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 33 1 30 27
        // the copy at BCI 19 has no proved local assignment
        // @bytecode 34 1
        // the copy at BCI 19 has no proved local assignment
    }

    // jarde: generic Signature projection refused for `collections()[Ljava/util/Collection;`: unsupported (ordinary_generic_source_unproved): method body or no-body declaration has no complete source proof
    public static java.util.Collection[] collections() {
        // jarde: not recovered: the recovery run for `collections()[Ljava/util/Collection;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method collections()[Ljava/util/Collection;
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
        // @bytecode 28 1 25 22 11 10 14 15 16 18 21
        // the copy at BCI 4 has no proved local assignment
        // @bytecode 29 1
        // the instruction at BCI 29 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 31
        // the instruction at BCI 31 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 34
        // the instruction at BCI 34 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 53 1 50 47 36 35 39 40 41 43 46
        // the copy at BCI 29 has no proved local assignment
        // @bytecode 54 1
        // the copy at BCI 29 has no proved local assignment
    }

    public static java.lang.Throwable[] failures() {
        // jarde: not recovered: the recovery run for `failures()[Ljava/lang/Throwable;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method failures()[Ljava/lang/Throwable;
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
        // @bytecode 18 1 15 12
        // the copy at BCI 4 has no proved local assignment
        // @bytecode 19 1
        // the instruction at BCI 19 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 21
        // the instruction at BCI 21 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 24
        // the instruction at BCI 24 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 33 1 30 27
        // the copy at BCI 19 has no proved local assignment
        // @bytecode 34 1
        // the copy at BCI 19 has no proved local assignment
    }

    public static Base[] ownDirect() {
        // jarde: not recovered: the recovery run for `ownDirect()[LBase;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method ownDirect()[LBase;
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
        // @bytecode 18 1 15 12
        // the copy at BCI 4 has no proved local assignment
        // @bytecode 19 1
        // the instruction at BCI 19 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 21
        // the instruction at BCI 21 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 24
        // the instruction at BCI 24 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 33 1 30 27
        // the copy at BCI 19 has no proved local assignment
        // @bytecode 34 1
        // the copy at BCI 19 has no proved local assignment
    }

    public static Base[] ownTwoHop() {
        // jarde: not recovered: the recovery run for `ownTwoHop()[LBase;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method ownTwoHop()[LBase;
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
        // @bytecode 18 1 15 12
        // the copy at BCI 4 has no proved local assignment
        // @bytecode 19 1
        // the instruction at BCI 19 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 21
        // the instruction at BCI 21 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 24
        // the instruction at BCI 24 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 33 1 30 27
        // the copy at BCI 19 has no proved local assignment
        // @bytecode 34 1
        // the copy at BCI 19 has no proved local assignment
    }

    public static LocalInterface[] ownInterface() {
        // jarde: not recovered: the recovery run for `ownInterface()[LLocalInterface;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method ownInterface()[LLocalInterface;
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
        // @bytecode 18 1 15 12
        // the copy at BCI 4 has no proved local assignment
        // @bytecode 19 1
        // the instruction at BCI 19 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 21
        // the instruction at BCI 21 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 24
        // the instruction at BCI 24 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 33 1 30 27
        // the copy at BCI 19 has no proved local assignment
        // @bytecode 34 1
        // the copy at BCI 19 has no proved local assignment
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
        java.lang.Object local2 = arg0[1];
        if (local2 == null) {
            java.lang.System.out.println("null");
        } else {
            java.lang.System.out.println((java.lang.String) local2.getClass().getName());
        }
        return;
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `Main`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        observe((java.lang.Object[]) sequence());
        observe((java.lang.Object[]) collections());
        observe((java.lang.Object[]) failures());
        observe((java.lang.Object[]) ownDirect());
        observe((java.lang.Object[]) ownTwoHop());
        observe((java.lang.Object[]) ownInterface());
        return;
    }
}

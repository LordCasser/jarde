// jarde: presentation of `BoundaryControls` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class BoundaryControls extends java.lang.Object {
    public BoundaryControls() {
        // @method <init>()V
        // @declaration a constructor of `BoundaryControls`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static java.lang.String mark(java.lang.String arg0) {
        // @method mark(Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `BoundaryControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.print("mark:");
        java.lang.System.out.println(arg0);
        return arg0;
    }

    static int number(java.lang.String arg0) {
        // @method number(Ljava/lang/String;)I
        // @declaration a static method of `BoundaryControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        mark(arg0);
        return 1;
    }

    static java.lang.Object[] firstThenUnsupportedStructure() {
        // @method firstThenUnsupportedStructure()[Ljava/lang/Object;
        // @declaration a static method of `BoundaryControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return new java.lang.Object[]{new java.lang.StringBuilder((java.lang.String) mark("first")), new java.lang.Long((long) number("second"))};
    }

    static void old(java.lang.Object[] arg0) {
        // jarde: not recovered: the recovery run for `old([Ljava/lang/Object;)V` produced no statement (explanation only); the artifact's own comment lines are below
        // @method old([Ljava/lang/Object;)V
        // @declaration a static method of `BoundaryControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 2
        // the instruction at BCI 2 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 5
        // the instruction at BCI 5 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 14 11 8 0 1 2 5 6 15
        // the copy at BCI 5 has no proved local assignment
        jarde_refused_body();
    }

    static java.lang.Object[] repeated() {
        // jarde: not recovered: the recovery run for `repeated()[Ljava/lang/Object;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method repeated()[Ljava/lang/Object;
        // @declaration a static method of `BoundaryControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 7
        // the instruction at BCI 7 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 10
        // the instruction at BCI 10 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 19 16 13 5 0 1 4 6 7 10 11 20 21 22 25 26 28 31 34 35 36
        // the copy at BCI 10 has no proved local assignment
        // @bytecode 22
        // the instruction at BCI 22 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 25
        // the instruction at BCI 25 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 34 31 28 20
        // the copy at BCI 25 has no proved local assignment
    }

    static java.lang.Object[] descending() {
        // jarde: not recovered: the recovery run for `descending()[Ljava/lang/Object;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method descending()[Ljava/lang/Object;
        // @declaration a static method of `BoundaryControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 7
        // the instruction at BCI 7 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 10
        // the instruction at BCI 10 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 19 16 13 5 0 1 4 6 7 10 11 20 21 22 25 26 28 31 34 35 36
        // the copy at BCI 10 has no proved local assignment
        // @bytecode 22
        // the instruction at BCI 22 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 25
        // the instruction at BCI 25 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 34 31 28 20
        // the copy at BCI 25 has no proved local assignment
    }

    static java.lang.Object[] crossBlock(boolean arg0) {
        // @method crossBlock(Z)[Ljava/lang/Object;
        // @declaration a static method of `BoundaryControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.Object[] local1;
        local1 = new java.lang.Object[1];
        java.lang.StringBuilder local2 = new java.lang.StringBuilder((java.lang.String) mark("cross-block"));
        if (arg0) {
            local1[0] = local2;
        }
        return local1;
    }

    static java.lang.Number[] closedNumberBoundary() {
        // @method closedNumberBoundary()[Ljava/lang/Number;
        // @declaration a static method of `BoundaryControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return new java.lang.Number[]{new java.lang.Integer((java.lang.String) mark("1")), new java.math.BigDecimal((java.lang.String) mark("2"))};
    }
}

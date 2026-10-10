// jarde: presentation of `jadx/tests/integration/switches/TestSwitchNoDefault$TestCls` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package jadx.tests.integration.switches;

public class TestSwitchNoDefault$TestCls extends java.lang.Object {
    public TestSwitchNoDefault$TestCls() {
        // @method <init>()V
        // @declaration a constructor of `jadx.tests.integration.switches.TestSwitchNoDefault$TestCls`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public void test(int a) {
        // jarde: not recovered: the recovery run for `test(I)V` produced no statement (explanation only); the artifact's own comment lines are below
        // @method test(I)V
        // @declaration an instance method of `jadx.tests.integration.switches.TestSwitchNoDefault$TestCls`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 57 53 56 0 1 2 3 32 34 35 38 40 41 44 46 47 50 52 60
        // the parameter 0 of the invocation at BCI 57 is declared `java.lang.String` presents `Object` but the invocation requires `java.lang.String` and this layer has no safe reference conversion evidence
        jarde_refused_body();
    }
}

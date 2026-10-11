// jarde: presentation of `jadx/tests/integration/switches/TestSwitchWithFallThroughCase2$TestCls` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package jadx.tests.integration.switches;

public class TestSwitchWithFallThroughCase2$TestCls extends java.lang.Object {
    public TestSwitchWithFallThroughCase2$TestCls() {
        // @method <init>()V
        // @declaration a constructor of `jadx.tests.integration.switches.TestSwitchWithFallThroughCase2$TestCls`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public java.lang.String test(int a, boolean b, boolean c) {
        // jarde: not recovered: the recovery run for `test(IZZ)Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method test(IZZ)Ljava/lang/String;
        // @declaration an instance method of `jadx.tests.integration.switches.TestSwitchWithFallThroughCase2$TestCls`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 2 4 5 8 9 10 11 36 39 40 43 45 48 50 53 56 58 59 60 63 64 67 68 71 74 75 78 80 83 85 88 91 93 96 99 100 103 105 108 110 113 116 118 121 122 125 128 129 132 134 137 139 142 145 147 150 153 156 157 160 162 165 167 170 173 175 178 179 182 184 187 189 192 195 197 198 201 202 205 208 209 212 214 217 219 222 225 227 229
        // the arms of the branch in block 0 do not meet at one join
    }

    public void check() {
        // @method check()V
        // @declaration an instance method of `jadx.tests.integration.switches.TestSwitchWithFallThroughCase2$TestCls`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        org.assertj.core.api.Assertions.assertThat((java.lang.String) this.test(5, true, true)).isEqualTo(">1+-");
        org.assertj.core.api.Assertions.assertThat((java.lang.String) this.test(1, true, true)).isEqualTo(">2+-");
        org.assertj.core.api.Assertions.assertThat((java.lang.String) this.test(3, true, true)).isEqualTo("+-");
        org.assertj.core.api.Assertions.assertThat((java.lang.String) this.test(16, true, true)).isEqualTo("default+-");
        org.assertj.core.api.Assertions.assertThat((java.lang.String) this.test(-1, true, true)).isEqualTo("-");
        return;
    }
}

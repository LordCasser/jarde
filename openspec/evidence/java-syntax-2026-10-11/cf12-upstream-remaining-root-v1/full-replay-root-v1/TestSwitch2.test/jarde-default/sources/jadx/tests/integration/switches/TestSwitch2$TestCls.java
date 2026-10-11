// jarde: presentation of `jadx/tests/integration/switches/TestSwitch2$TestCls` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package jadx.tests.integration.switches;

public class TestSwitch2$TestCls extends java.lang.Object {
    boolean isLongtouchable;

    boolean isMultiTouchZoom;

    boolean isCanZoomIn;

    boolean isCanZoomOut;

    boolean isScrolling;

    float multiTouchZoomOldDist;

    public TestSwitch2$TestCls() {
        // @method <init>()V
        // @declaration a constructor of `jadx.tests.integration.switches.TestSwitch2$TestCls`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public void test(int action) {
        // jarde: not recovered: the recovery run for `test(I)V` produced no statement (explanation only); the artifact's own comment lines are below
        // @method test(I)V
        // @declaration an instance method of `jadx.tests.integration.switches.TestSwitch2$TestCls`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 1 4 5
        // two switch arms of the block at BCI 0 claim the same block, so one case falls through into another case's code
        // @bytecode 48 49 50 53 56 57 60 63 64 65 68 71 72 75 78 79 82 83 84 85 88 89 92 94 95 98 99 100 103 104 105 106 107 108 111 112 115 118 119 120 121 124 125 128 131 132 133 136 137 138 139 140 143 144 147 149 150 153 154 155 158 159 160 163 164 165 168 171 172 173 176 177 178 181
        // 17 live block(s) are reachable only through edges the normal-flow view leaves out: [48, 56, 71, 137, 164, 63, 78, 153, 171, 181, 98, 136, 176, 111, 118, 131, 124]
        jarde_refused_body();
    }
}

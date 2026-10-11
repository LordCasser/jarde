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
        // @method test(I)V
        // @declaration an instance method of `jadx.tests.integration.switches.TestSwitch2$TestCls`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        switch (action & 255) {
            case 0:
                this.isLongtouchable = true;
                break;
            case 1:
            case 6:
                if (this.isMultiTouchZoom) {
                    this.isMultiTouchZoom = false;
                }
                break;
            case 2:
                if (this.isMultiTouchZoom) {
                    float dist = this.multiTouchZoomOldDist;
                    if (java.lang.Math.abs(dist - this.multiTouchZoomOldDist) > 0x1.400000p3f) {
                        float scale = dist / this.multiTouchZoomOldDist;
                        if (scale > 0x1.000000p0f && this.isCanZoomIn || scale < 0x1.000000p0f && this.isCanZoomOut) {
                            this.multiTouchZoomOldDist = dist;
                        }
                    }
                    return;
                }
                break;
            case 3:
            case 4:
                break;
            case 5:
                this.multiTouchZoomOldDist = (float) action;
                if (this.multiTouchZoomOldDist > 0x1.400000p3f) {
                    this.isMultiTouchZoom = true;
                    this.isLongtouchable = false;
                    return;
                }
                break;
        }
        if (this.isScrolling) {
            if (action == 1) {
                this.isScrolling = false;
            }
        }
        return;
    }
}

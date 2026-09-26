import java.util.function.IntSupplier;

public class OuterSuperCases extends BaseA {
    final int state;
    OuterSuperCases(int state) { this.state = state; }
    @Override int value() { return state + 100; }

    class Member extends MemberBase {
        int explicitOther(OuterSuperCases other) { return other.value(); }
        int capturedOuterCall() { return OuterSuperCases.super.value(); }
        int secondCapturedCall() { return OuterSuperCases.super.value() + 1; }
        int otherBridgeCandidate(OuterSuperCases other) { return OuterSuperCases.super.value(); }
        IntSupplier lambdaOuterCall() { return () -> OuterSuperCases.super.value(); }
        int ownCall() { return this.value(); }
    }

    public static void main(String[] args) {
        OuterSuperCases outer = new OuterSuperCases(10);
        OuterSuperCases other = new OuterSuperCases(20);
        Member m = outer.new Member();
        System.out.println(m.explicitOther(other) + ":" + m.capturedOuterCall() + ":"
                + m.secondCapturedCall() + ":" + m.otherBridgeCandidate(other) + ":"
                + m.lambdaOuterCall().getAsInt() + ":" + m.ownCall());
    }
}
class BaseA { int value() { return ((OuterSuperCases)this).state; } }
class BaseB { int value() { return 200; } }
class MemberBase { int value() { return 3; } }
